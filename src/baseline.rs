use crate::config::Config;
use crate::http::HttpClient;
use futures::{StreamExt, stream::FuturesUnordered};

pub async fn baseline_available() -> bool {
    let config = Config::load().expect("Failed to load config");
    check_websites(&config.baseline_sites, reqwest::Client::new()).await
}

pub async fn check_websites<T: HttpClient + 'static>(websites: &[String], client: T) -> bool {
    // Early return for empty list
    if websites.is_empty() {
        return false;
    }

    // For single website, direct await is most efficient
    if websites.len() == 1 {
        return client.check_url(&websites[0]).await;
    }

    // Use FuturesUnordered for efficient concurrent execution with early termination
    let mut futures = websites
        .iter()
        .map(|site| {
            let client = client.clone();
            async move { client.check_url(site).await }
        })
        .collect::<FuturesUnordered<_>>();

    // Return true as soon as any future resolves to true
    while let Some(result) = futures.next().await {
        if result {
            return true;
        }
    }

    false
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
    }

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn check_url(&self, _url: &str) -> bool {
            self.should_succeed.load(Ordering::SeqCst)
        }

        async fn get_status_code(&self, _url: &str) -> Result<u16, reqwest::Error> {
            Ok(200)
        }
    }

    #[tokio::test]
    async fn test_check_websites_success() {
        let client = MockHttpClient {
            should_succeed: Arc::new(AtomicBool::new(true)),
        };
        let websites = vec!["https://test.com".to_string()];

        assert!(check_websites(&websites, client).await);
    }

    #[tokio::test]
    async fn test_check_websites_failure() {
        let client = MockHttpClient {
            should_succeed: Arc::new(AtomicBool::new(false)),
        };
        let websites = vec!["https://test.com".to_string()];

        assert!(!check_websites(&websites, client).await);
    }

    #[tokio::test]
    async fn test_check_websites_with_real_config() {
        let config = Config::load().expect("Failed to load config");
        let client = reqwest::Client::new();
        let result = check_websites(&config.baseline_sites, client).await;
        println!("Completed without panicking. Available: {}", result);
    }

    #[tokio::test]
    async fn test_check_websites_with_fake_websites() {
        let fake_websites = vec![
            "https://definitely-not-a-real-website-12345.com".to_string(),
            "https://another-fake-site-67890.net".to_string(),
            "https://nonexistent-domain-xyz.org".to_string(),
        ];
        let client = reqwest::Client::new();

        let result = check_websites(&fake_websites, client).await;
        assert!(!result, "Fake websites should not be reachable");
    }

    #[tokio::test]
    async fn test_check_websites_empty_list() {
        let empty_websites: Vec<String> = vec![];
        let client = reqwest::Client::new();

        let result = check_websites(&empty_websites, client).await;
        assert!(!result, "Empty websites list should return false");
    }
}
