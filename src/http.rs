use crate::{baseline::baseline_available, schema::Website};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use std::time::SystemTime;
use reqwest::Client;

// Trait for HTTP clients to enable testing
#[async_trait::async_trait]
pub trait HttpClient: Clone + Send + Sync {
    async fn check_url(&self, url: &str) -> bool;
    async fn get_status_code(&self, url: &str) -> Result<u16, reqwest::Error>;
}

// Implementation for reqwest::Client
#[async_trait::async_trait]
impl HttpClient for reqwest::Client {
    async fn check_url(&self, url: &str) -> bool {
        let res = self.get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send().await;
        res.is_ok()
    }

    async fn get_status_code(&self, url: &str) -> Result<u16, reqwest::Error> {
        let res = self.get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send().await?;
        Ok(res.status().as_u16())
    }
}

// Function to update HTTP status of each website
pub async fn update_http_status(webs: &mut [Website]) {
    // Check baseline availability
    let result = baseline_available().await;
    if !result {
        return;
    }

    // Create a vector to store all the futures
    let futures: Vec<_> = webs.iter_mut().map(update_web_status).collect();

    // Wait for all futures to complete
    join_all(futures).await;
}

async fn update_web_status(web: &mut Website) {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    web.last_checked_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    let client = Client::new();
    match client.get_status_code(&web.url).await {
        Ok(status) => web.status = status as i32,
        Err(_e) => web.status = 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock HTTP client for testing
    #[derive(Clone)]
    struct MockHttpClient {
        should_succeed: Arc<AtomicBool>,
        status_code: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn check_url(&self, _url: &str) -> bool {
            self.should_succeed.load(Ordering::SeqCst)
        }

        async fn get_status_code(&self, _url: &str) -> Result<u16, reqwest::Error> {
            if self.status_code.load(Ordering::SeqCst) {
                Ok(200)
            } else {
                // Simulate a network error by using an invalid URL
                let client = reqwest::Client::new();
                client.get_status_code("invalid://url").await
            }
        }
    }

    #[tokio::test]
    async fn test_get_status_success_mock() {
        let client = MockHttpClient {
            should_succeed: Arc::new(AtomicBool::new(true)),
            status_code: Arc::new(AtomicBool::new(true)),
        };

        let result = client.get_status_code("https://example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_get_status_failure_mock() {
        let client = MockHttpClient {
            should_succeed: Arc::new(AtomicBool::new(false)),
            status_code: Arc::new(AtomicBool::new(false)),
        };

        let result = client.get_status_code("https://example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_status_real_success() {
        let client = reqwest::Client::new();
        let result = client.get_status_code("https://www.google.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
    }

    #[tokio::test]
    async fn test_get_status_real_failure() {
        let client = reqwest::Client::new();
        let result = client
            .get_status_code(
                "https://this-is-a-fake-website-that-does-not-exist.com",
            )
            .await;
        assert!(result.is_err());
    }
}
