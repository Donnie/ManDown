use crate::alert::alert_users;
use crate::baseline::baseline_available;
use crate::http::{find_changed_websites, get_status};
use crate::mongo::{Website, get_sites, update_db};
use futures::future::join_all;
use mongodb::Collection;
use mongodb::bson::Document;
use std::sync::Arc;
use teloxide::Bot;
use tokio::time;

pub fn start_downtime_checker(
    bot: Bot,
    collection: Arc<Collection<Document>>,
    http_client: Arc<reqwest::Client>,
) {
    let interval = dotenvy::var("FREQ")
        .unwrap_or("600".to_string())
        .parse()
        .expect("FREQ must be a number");

    tokio::spawn(async move {
        downtime_check(&collection, interval, bot, http_client).await;
    });
}

async fn downtime_check(
    collection: &Collection<Document>,
    interval: u64,
    bot: Bot,
    client: Arc<reqwest::Client>,
) {
    loop {
        log::info!("Starting downtime check");
        let changed_websites = get_changed_sites(collection, client.clone()).await;
        log::info!("Found {} changed websites", changed_websites.len());
        handle_changed_websites(collection, bot.clone(), &changed_websites).await;
        time::sleep(time::Duration::from_secs(interval)).await;
    }
}

async fn get_changed_sites(
    collection: &Collection<Document>,
    client: Arc<reqwest::Client>,
) -> Vec<Website> {
    if !baseline_available(client.clone()).await {
        log::info!("Baseline not available, skipping check.");
        return Vec::new();
    }

    let mut all_changed_websites = Vec::new();
    let mut skip = 0;
    const LIMIT: i64 = 20;

    loop {
        let websites = match get_sites(collection, skip, LIMIT).await {
            Ok(sites) => sites,
            Err(e) => {
                log::error!("Error getting websites from DB: {e}");
                return Vec::new();
            }
        };

        if websites.is_empty() {
            break;
        }

        log::info!("Getting statuses for {} websites", websites.len());

        let new_statuses = fetch_website_statuses(&websites, client.clone()).await;
        let changed_in_batch = find_changed_websites(&websites, &new_statuses);
        all_changed_websites.extend(changed_in_batch);

        skip += LIMIT as u64;
    }

    all_changed_websites
}

async fn fetch_website_statuses(websites: &[Website], client: Arc<reqwest::Client>) -> Vec<u16> {
    let status_futures = websites.iter().map(|web| get_status(&client, &web.url));
    join_all(status_futures).await
}

async fn handle_changed_websites(
    collection: &Collection<Document>,
    bot: Bot,
    changed_websites: &[Website],
) {
    let web_count = changed_websites.len();

    if web_count == 0 {
        return;
    }

    alert_users(bot, changed_websites).await;

    if let Err(e) = update_db(collection, changed_websites).await {
        log::error!("Error updating websites in DB: {e}");
    }
}
