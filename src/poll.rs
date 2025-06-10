use crate::alert::alert_users;
use crate::baseline::baseline_available;
use crate::http::{find_changed_websites, get_status};
use crate::mongo::{Website, get_sites, update_db};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use log::{error, info};
use mongodb::Collection;
use mongodb::bson::Document;
use std::sync::Arc;
use std::time::SystemTime;
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
        let changed_websites = get_changed_sites(collection, client.clone()).await;
        handle_changed_websites(collection, bot.clone(), &changed_websites).await;
        time::sleep(time::Duration::from_secs(interval)).await;
    }
}

async fn get_changed_sites(
    collection: &Collection<Document>,
    client: Arc<reqwest::Client>,
) -> Vec<Website> {
    println!("Checking Websites now");

    if !baseline_available(client.clone()).await {
        println!("Baseline not available, skipping check.");
        return Vec::new();
    }

    let mut all_changed_websites = Vec::new();
    let mut skip = 0;
    const LIMIT: i64 = 20;

    loop {
        println!(
            "{}: Getting websites from DB, skip: {}",
            DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%d %H:%M:%S"),
            skip
        );
        let websites = match get_sites(collection, skip, LIMIT).await {
            Ok(sites) => sites,
            Err(e) => {
                error!("Error getting websites from DB: {}", e);
                return Vec::new();
            }
        };
        println!(
            "{}: Got {} websites from DB",
            DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%d %H:%M:%S"),
            websites.len()
        );

        if websites.is_empty() {
            break;
        }

        let new_statuses = fetch_website_statuses(&websites, client.clone()).await;
        let changed_in_batch = find_changed_websites(&websites, &new_statuses);
        all_changed_websites.extend(changed_in_batch);

        skip += LIMIT as u64;
        println!(
            "{}: Skipping {} websites",
            DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%d %H:%M:%S"),
            skip
        );
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
    info!("{} websites changed", web_count);

    if web_count == 0 {
        return;
    }

    alert_users(bot, changed_websites).await;

    if let Err(e) = update_db(collection, changed_websites).await {
        error!("Error updating websites in DB: {}", e);
    }
}
