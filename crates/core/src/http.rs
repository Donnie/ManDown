use crate::mongo::Website;
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::{sync::Arc, time::SystemTime};

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
pub fn cust_client(timeout: u64) -> Arc<Client> {
    Arc::new(
        Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .expect("Failed to build HTTP client"),
    )
}

// Function to update HTTP status of each website
pub async fn get_status(client: &Client, url: &str) -> u16 {
    let status = client.get_status_code(url).await;

    // If status is 0, retry once
    if status == 0 {
        log::info!("Retrying {url} because status is 0");
        client.get_status_code(url).await
    } else {
        status
    }
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
        let client = cust_client(5);
        let result = client.get_status_code("https://www.google.com").await;
        assert_eq!(result, 200);
    }

    #[tokio::test]
    async fn test_get_status_real_failure() {
        let client = cust_client(5);
        let result = client
            .get_status_code("https://this-is-a-fake-website-that-does-not-exist.com")
            .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_get_status_real_website() {
        let client = cust_client(5);
        let status = get_status(&client, "https://www.google.com").await;
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_get_status_fake_website() {
        let client = cust_client(5);
        let status = get_status(
            &client,
            "https://this-is-a-fake-website-that-does-not-exist-123456789.com",
        )
        .await;
        assert_eq!(status, 0);
    }

    #[tokio::test]
    async fn test_get_status_timeout() {
        let client = cust_client(1); // 1 second timeout for faster test
        let status = get_status(&client, "http://10.255.255.1:80").await; // Non-routable IP
        assert_eq!(status, 0);
    }

    #[tokio::test]
    async fn test_get_status_invalid_url() {
        let client = cust_client(5);
        let status = get_status(&client, "not-a-valid-url").await;
        assert_eq!(status, 0);
    }

    use super::find_changed_websites;
    use crate::mongo::Website;
    use mongodb::bson::oid::ObjectId;

    #[test]
    fn test_find_changed_websites_multiple_changes() {
        let random_date = "2024-01-01 00:00:00".to_string();
        // Original websites
        let original_websites = vec![
            Website {
                id: Some(ObjectId::new()),
                last_updated: random_date.clone(),
                telegram_id: "1234567890".to_string(),
                status: 200,
                url: "https://example1.com".to_string(),
            },
            Website {
                id: Some(ObjectId::new()),
                last_updated: random_date.clone(),
                telegram_id: "1234567890".to_string(),
                status: 404,
                url: "https://example2.com".to_string(),
            },
            Website {
                id: Some(ObjectId::new()),
                last_updated: random_date.clone(),
                telegram_id: "1234567890".to_string(),
                status: 500,
                url: "https://example3.com".to_string(),
            },
            Website {
                id: Some(ObjectId::new()),
                last_updated: random_date.clone(),
                telegram_id: "1234567890".to_string(),
                status: 0,
                url: "https://example4.com".to_string(),
            },
            Website {
                id: Some(ObjectId::new()),
                last_updated: random_date.clone(),
                telegram_id: "1234567890".to_string(),
                status: 200,
                url: "https://example5.com".to_string(),
            },
        ];

        // New statuses derived from what would be updated websites
        let new_statuses = vec![
            200, // unchanged
            200, // changed from 404
            503, // changed from 500
            200, // changed from 0
            200, // unchanged
        ];

        let result = find_changed_websites(&original_websites, &new_statuses);

        // Should only include websites with changed status
        assert_eq!(result.len(), 3);

        // Verify the correct websites were identified as changed
        let changed_urls: Vec<String> = result.iter().map(|w| w.url.clone()).collect();
        assert!(changed_urls.contains(&"https://example2.com".to_string())); // Website 2 changed from 404 to 200
        assert!(changed_urls.contains(&"https://example3.com".to_string())); // Website 3 changed from 500 to 503
        assert!(changed_urls.contains(&"https://example4.com".to_string())); // Website 4 changed from 0 to 200

        // Verify unchanged websites are not included
        assert!(!changed_urls.contains(&"https://example1.com".to_string())); // Website 1 unchanged
        assert!(!changed_urls.contains(&"https://example5.com".to_string())); // Website 5 unchanged

        // Verify that the changed websites have updated timestamps
        for web in result {
            assert_ne!(web.last_updated, random_date);
        }
    }

    #[test]
    fn test_find_changed_websites_no_changes() {
        let original_websites = vec![Website {
            id: Some(ObjectId::new()),
            last_updated: "2024-01-01 00:00:00".to_string(),
            telegram_id: "123".to_string(),
            status: 200,
            url: "https://example1.com".to_string(),
        }];
        let new_statuses = vec![200];
        let changed = find_changed_websites(&original_websites, &new_statuses);
        assert!(changed.is_empty());
    }
}
