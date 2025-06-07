use crate::schema::Website;
use chrono::{DateTime, Utc};
use futures::future::join_all;
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
            Err(_) => 0
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

// Function to update HTTP status of each website
pub async fn update_http_statuses(webs: &mut [Website], client: &Client) {
    // Create a vector to store all the futures
    let futures: Vec<_> = webs
        .iter_mut()
        .map(|web| double_check_http_status(web, client))
        .collect();

    // Wait for all futures to complete
    join_all(futures).await;
}

async fn update_http_status(web: &mut Website, client: &Client) {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    web.last_checked_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    web.status = client.get_status_code(&web.url).await as i32;
}

// Function to update HTTP status with retry for failed checks
async fn double_check_http_status(web: &mut Website, client: &Client) {
    // First attempt
    update_http_status(web, client).await;

    // If status is 0, retry once
    if web.status == 0 {
        update_http_status(web, client).await;
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

        async fn get_status_code(&self, _url: &str) -> u16 {
            if self.status_code.load(Ordering::SeqCst) {
                200
            } else {
                0
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
        assert_eq!(result, 200);
    }

    #[tokio::test]
    async fn test_get_status_failure_mock() {
        let client = MockHttpClient {
            should_succeed: Arc::new(AtomicBool::new(false)),
            status_code: Arc::new(AtomicBool::new(false)),
        };

        let result = client.get_status_code("https://example.com").await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_get_status_real_success() {
        let client = reqwest::Client::new();
        let result = client.get_status_code("https://www.google.com").await;
        assert_eq!(result, 200);
    }

    #[tokio::test]
    async fn test_get_status_real_failure() {
        let client = reqwest::Client::new();
        let result = client.get_status_code("https://this-is-a-fake-website-that-does-not-exist.com").await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_update_http_status_real_website() {
        let mut website = Website {
            id: 1,
            last_checked_time: "2020-01-01 00:00:00".to_string(),
            status: 0,
            url: "https://www.google.com".to_string(),
        };

        let client = cust_client(5);
        update_http_status(&mut website, &client).await;

        // Check that the timestamp was updated (should not be the old value)
        assert_ne!(website.last_checked_time, "2020-01-01 00:00:00");

        // Check that status was updated to a valid HTTP status code (200)
        assert_eq!(website.status, 200);
    }

    #[tokio::test]
    async fn test_update_http_status_fake_website() {
        let mut website = Website {
            id: 2,
            last_checked_time: "2020-01-01 00:00:00".to_string(),
            status: 200,
            url: "https://this-is-a-fake-website-that-does-not-exist-123456789.com".to_string(),
        };

        let client = cust_client(5);
        update_http_status(&mut website, &client).await;

        // Check that the timestamp was updated
        assert_ne!(website.last_checked_time, "2020-01-01 00:00:00");

        // Check that status was set to 0 (indicating failure)
        assert_eq!(website.status, 0);
    }

    #[tokio::test]
    async fn test_update_http_status_timeout() {
        let mut website = Website {
            id: 3,
            last_checked_time: "2020-01-01 00:00:00".to_string(),
            status: 200,
            url: "http://10.255.255.1:80".to_string(), // Non-routable IP that will timeout
        };

        let client = cust_client(5);
        update_http_status(&mut website, &client).await;

        // Check that the timestamp was updated
        assert_ne!(website.last_checked_time, "2020-01-01 00:00:00");

        // Check that status was set to 0 (indicating timeout/failure)
        assert_eq!(website.status, 0);
    }

    #[tokio::test]
    async fn test_update_http_status_invalid_url() {
        let mut website = Website {
            id: 4,
            last_checked_time: "2020-01-01 00:00:00".to_string(),
            status: 200,
            url: "not-a-valid-url".to_string(),
        };

        let client = cust_client(5);
        update_http_status(&mut website, &client).await;

        // Check that the timestamp was updated
        assert_ne!(website.last_checked_time, "2020-01-01 00:00:00");

        // Check that status was set to 0 (indicating failure)
        assert_eq!(website.status, 0);
    }

    #[tokio::test]
    async fn test_update_http_statuses() {
        let random_date = "2020-01-01 00:00:00".to_string();
        let mut websites = vec![
            Website {
                id: 1,
                last_checked_time: random_date.clone(),
                status: 0,
                url: "https://www.google.com".to_string(),
            },
            Website {
                id: 2,
                last_checked_time: random_date.clone(),
                status: 20,
                url: "https://this-is-a-fake-website-that-does-not-exist-123456789.com".to_string(),
            },
            Website {
                id: 3,
                last_checked_time: random_date.clone(),
                status: 30,
                url: "not-a-valid-url".to_string(),
            },
            Website {
                id: 4,
                last_checked_time: random_date.clone(),
                status: 200,
                url: "http://10.255.255.1:80".to_string(), // Non-routable IP that will timeout
            },
        ];

        let client = cust_client(5);
        update_http_statuses(&mut websites, &client).await;

        // Check that all timestamps were updated
        for website in &websites {
            assert_ne!(website.last_checked_time, random_date);
        }

        // Check specific status codes
        assert_eq!(websites[0].status, 200); // Google should be accessible
        assert_eq!(websites[1].status, 0); // Fake website should fail
        assert_eq!(websites[2].status, 0); // Invalid URL should fail
        assert_eq!(websites[3].status, 0); // Timeout should fail
    }
}
