use crate::mongo::Website;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::time::SystemTime;

// Trait for HTTP clients to enable testing
#[async_trait::async_trait]
pub trait HttpClient: Clone + Send + Sync {
    async fn check_url(&self, url: &str) -> bool;
    async fn get_status_code(&self, url: &str) -> u16;
}

// Implementation for reqwest::Client
#[async_trait::async_trait]
impl HttpClient for reqwest::Client {
    async fn check_url(&self, url: &str) -> bool {
        let res = self.get(url).send().await;
        res.is_ok()
    }

    async fn get_status_code(&self, url: &str) -> u16 {
        match self.get(url).send().await {
            Ok(res) => res.status().as_u16(),
            Err(_) => 0,
        }
    }
}

// Function to create a client with preset timeout
pub fn cust_client(timeout: u64) -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .expect("Failed to build HTTP client")
}

pub fn find_changed_websites(original_webs: &[Website], new_statuses: &[u16]) -> Vec<Website> {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    let timestamp = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    original_webs
        .iter()
        .zip(new_statuses.iter())
        .filter_map(|(web, &new_status)| {
            if web.status != new_status as i32 {
                let mut updated_web = web.clone();
                updated_web.status = new_status as i32;
                updated_web.last_updated = timestamp.clone();
                Some(updated_web)
            } else {
                None
            }
        })
        .collect()
}

// Function to get HTTP status of each website
pub async fn get_status(client: &Client, url: &str) -> u16 {
    let status = client.get_status_code(url).await;

    // If status is 0, retry once
    if status == 0 {
        client.get_status_code(url).await
    } else {
        status
    }
}
