use crate::config::Config;
use crate::http::HttpClient;
use futures::future::join_all;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn baseline_available() -> bool {
  let config = Config::load().expect("Failed to load config");  
  check_websites(&config.baseline_sites, reqwest::Client::new()).await
}

pub async fn check_websites<T: HttpClient>(websites: &[String], client: T) -> bool {
  let tasks: Vec<_> = websites
      .iter()
      .map(|site| {
          let client = client.clone();
          let site = site.clone();
          tokio::spawn(async move { client.check_url(&site).await })
      })
      .collect();

  let results: Vec<_> = join_all(tasks).await;
  results.into_iter().any(|res| res.unwrap_or(false))
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::Arc;

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
}
