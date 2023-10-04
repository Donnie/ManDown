use crate::schema::Website;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use futures::future::join_all;

// Function to update HTTP status of each website
pub async fn update_http_status(webs: &mut Vec<Website>) {
    // Create a vector to store all the futures
    let futures: Vec<_> = webs.iter_mut()
      .map(|web| update_web_status(web))
      .collect();

    // Wait for all futures to complete
    join_all(futures).await;

    println!("Updated HTTP status for all websites");
}

async fn update_web_status(web: &mut Website) {
    let datetime: DateTime<Utc> = SystemTime::now().into();
    web.last_checked_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

    match get_status(&web.url).await {
        Ok(status) => web.status = status,
        Err(_e) => web.status = 0,
    }
}

async fn get_status(url: &str) -> Result<i32, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;

    Ok(res.status().as_u16() as i32)
}
