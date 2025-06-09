use crate::alert::{alert_users, notify_user};
use crate::baseline::baseline_available;
use crate::data::{get_all_websites, write_all_websites};
use crate::http::{cust_client, find_changed_websites, get_status, update_http_statuses};
use crate::mongo::{Website, get_all_sites, update_db};
use crate::schema::Website as SQLWebsite;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use futures::future::join_all;
use log::{error, info};
use mongodb::Collection;
use mongodb::bson::Document;
use teloxide::Bot;
use tokio::time;

pub async fn downtime_check(collection: &Collection<Document>, interval: u64, bot: Bot) {
    loop {
        let changed_websites = check_sites(collection).await;
        handle_changed_websites(collection, bot.clone(), &changed_websites).await;
        time::sleep(time::Duration::from_secs(interval)).await;
    }
}

async fn check_sites(collection: &Collection<Document>) -> Vec<Website> {
    info!("Checking Websites now");

    if !baseline_available().await {
        info!("Baseline not available, skipping check.");
        return Vec::new();
    }

    let websites = match get_all_sites(collection).await {
        Ok(sites) => {
            if sites.is_empty() {
                info!("No websites to check.");
                return Vec::new();
            }
            sites
        }
        Err(e) => {
            error!("Error getting websites from DB: {}", e);
            return Vec::new();
        }
    };

    let new_statuses = fetch_website_statuses(&websites).await;

    find_changed_websites(&websites, &new_statuses)
}

async fn fetch_website_statuses(websites: &[Website]) -> Vec<u16> {
    let client = cust_client(30);
    let status_futures = websites.iter().map(|web| get_status(&client, &web.url));
    join_all(status_futures).await
}

async fn handle_changed_websites(
    collection: &Collection<Document>,
    bot: Bot,
    changed_websites: &[Website],
) {
    let web_count = changed_websites.len();
    info!("{} websites changed", web_count);

    if web_count == 0 {
        return;
    }

    alert_users(bot, changed_websites).await;

    if let Err(e) = update_db(collection, changed_websites).await {
        error!("Error updating websites in DB: {}", e);
    }
}

pub async fn check_urls(
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
    interval: u64,
    bot: Bot,
) {
    loop {
        let pool = pool.clone();
        let bot = bot.clone();
        tokio::spawn(async move {
            let mut conn = pool.get().unwrap();
            check_websites(&mut conn, bot).await;
        });

        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

// Function to process DB
async fn check_websites(conn: &mut SqliteConnection, bot: Bot) {
    info!("Checking Websites now");

    // Check baseline availability
    let result = baseline_available().await;
    if !result {
        return;
    }

    // Read from DB
    let webs = get_all_websites(conn).expect("Error listing Websites");
    if webs.is_empty() {
        return;
    }

    // Create client
    let client = cust_client(30);

    // Clone websites for updating - we need the original statuses for comparison
    let mut updated_webs = webs.clone();

    // Update HTTP status of all websites in parallel
    update_http_statuses(&mut updated_webs, &client).await;

    // Find websites that have changed status
    let changed_webs = compare_websites(&webs, &updated_webs);

    let web_count = changed_webs.len();
    info!("{} websites changed", web_count);

    if web_count == 0 {
        return;
    }

    // Notify user if websites changed
    notify_user(conn, bot, changed_webs.clone()).await;

    // Write updated websites back to DB
    write_all_websites(conn, changed_webs).expect("Error updating Websites");
}

// Function to find websites that have changed status
fn compare_websites(original_webs: &[SQLWebsite], updated_webs: &[SQLWebsite]) -> Vec<SQLWebsite> {
    original_webs
        .iter()
        .zip(updated_webs.iter())
        .filter_map(|(original, updated)| {
            if original.status != updated.status {
                Some(updated.clone()) // Clone the updated website with new status
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::compare_websites;
    use crate::schema::Website;

    #[test]
    fn test_compare_websites_multiple_changes() {
        let random_date = "2024-01-01 00:00:00".to_string();
        // Original websites
        let original_websites = vec![
            Website {
                id: 1,
                last_checked_time: random_date.clone(),
                status: 200,
                url: "https://example1.com".to_string(),
            },
            Website {
                id: 2,
                last_checked_time: random_date.clone(),
                status: 404,
                url: "https://example2.com".to_string(),
            },
            Website {
                id: 3,
                last_checked_time: random_date.clone(),
                status: 500,
                url: "https://example3.com".to_string(),
            },
            Website {
                id: 4,
                last_checked_time: random_date.clone(),
                status: 0,
                url: "https://example4.com".to_string(),
            },
            Website {
                id: 5,
                last_checked_time: random_date.clone(),
                status: 200,
                url: "https://example4.com".to_string(),
            },
        ];

        // Updated websites with some status changes
        let updated_websites = vec![
            Website {
                id: 1,
                last_checked_time: random_date.clone(),
                status: 200, // unchanged
                url: "https://example1.com".to_string(),
            },
            Website {
                id: 2,
                last_checked_time: random_date.clone(),
                status: 200, // changed from 404
                url: "https://example2.com".to_string(),
            },
            Website {
                id: 3,
                last_checked_time: random_date.clone(),
                status: 503, // changed from 500
                url: "https://example3.com".to_string(),
            },
            Website {
                id: 4,
                last_checked_time: random_date.clone(),
                status: 200, // changed from 0
                url: "https://example4.com".to_string(),
            },
            Website {
                id: 5,
                last_checked_time: "2025-01-01 00:00:00".to_string(), // changed from 2024 to 2025
                status: 200,                                          // unchanged
                url: "https://example4.com".to_string(),
            },
        ];

        // Use the same logic as in check_websites function
        let result = compare_websites(&original_websites, &updated_websites);

        // Should only include websites with changed status
        assert_eq!(result.len(), 3);

        // Verify the correct websites were identified as changed
        let changed_ids: Vec<i32> = result.iter().map(|w| w.id).collect();
        assert!(changed_ids.contains(&2)); // Website 2 changed from 404 to 200
        assert!(changed_ids.contains(&3)); // Website 3 changed from 500 to 503
        assert!(changed_ids.contains(&4)); // Website 4 changed from 0 to 200

        // Verify unchanged websites are not included
        assert!(!changed_ids.contains(&1)); // Website 1 unchanged
        assert!(!changed_ids.contains(&5)); // Website 5 unchanged
    }
}
