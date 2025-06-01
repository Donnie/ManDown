use crate::schema::Website;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use std::time::SystemTime;
use async_trait;

// Trait for HTTP clients to enable testing
#[async_trait::async_trait]
pub trait HttpClient: Clone + Send + Sync {
    async fn check_url(&self, url: &str) -> bool;
}

// Implementation for reqwest::Client
#[async_trait::async_trait]
impl HttpClient for reqwest::Client {
    async fn check_url(&self, url: &str) -> bool {
        self.get(url).send().await.is_ok()
    }
}

// Function to update HTTP status of each website
pub async fn update_http_status(webs: &mut Vec<Website>) {
    // Check internet connection
    let result = has_internet_connection().await;
    if !result {
        return;
    }

    // Create a vector to store all the futures
    let futures: Vec<_> = webs.iter_mut().map(|web| update_web_status(web)).collect();

    // Wait for all futures to complete
    join_all(futures).await;
}

async fn update_web_status(web: &mut Website) {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    web.last_checked_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    match get_status(&web.url).await {
        Ok(status) => web.status = status as i32,
        Err(_e) => web.status = 0,
    }
}

pub async fn get_status(url: &str) -> Result<u16, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;

    Ok(res.status().as_u16())
}

async fn has_internet_connection() -> bool {
    let websites = [
        "https://www.facebook.com",
        "https://www.apple.com",
        "https://www.amazon.com",
        "https://www.netflix.com",
        "https://www.google.com",
    ];

    let client = reqwest::Client::new();

    let tasks: Vec<_> = websites
        .iter()
        .map(|&site| {
            let client = client.clone();
            tokio::spawn(async move { client.get(site).send().await.is_ok() })
        })
        .collect();

    let results: Vec<_> = join_all(tasks).await;

    results.into_iter().any(|res| res.unwrap_or(false))
}
