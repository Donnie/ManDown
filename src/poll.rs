use crate::alert::alert_users;
use crate::baseline::baseline_available;
use crate::http::{find_changed_websites, get_status};
use crate::mongo::{Website, get_all_sites, update_db};
use futures::future::join_all;
use log::{error, info};
use mongodb::Collection;
use mongodb::bson::Document;
use teloxide::Bot;
use tokio::time;
use std::sync::Arc;

pub async fn downtime_check(
    collection: &Collection<Document>,
    interval: u64,
    bot: Bot,
    client: Arc<reqwest::Client>,
) {
    loop {
        let changed_websites = check_sites(collection, client.clone()).await;
        handle_changed_websites(collection, bot.clone(), &changed_websites).await;
        time::sleep(time::Duration::from_secs(interval)).await;
    }
}

async fn check_sites(
    collection: &Collection<Document>,
    client: Arc<reqwest::Client>,
) -> Vec<Website> {
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

    let new_statuses = fetch_website_statuses(&websites, client).await;

    find_changed_websites(&websites, &new_statuses)
}

async fn fetch_website_statuses(
    websites: &[Website],
    client: Arc<reqwest::Client>,
) -> Vec<u16> {
    let status_futures = websites
        .iter()
        .map(|web| get_status(&client, &web.url));
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
