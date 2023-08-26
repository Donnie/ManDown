use crate::data::Record;
use futures::future::join_all;

// Function to update HTTP status of each website
pub async fn update_http_status(records: &mut Vec<Record>) {
    // Create a vector to store all the futures
    let futures: Vec<_> = records.iter_mut()
      .map(|record| update_record_status(record))
      .collect();

    // Wait for all futures to complete
    join_all(futures).await;
}

async fn update_record_status(record: &mut Record) {
    match get_status(&record.website).await {
        Ok(status) => record.status = status,
        Err(e) => handle_error(record, &e),
    }
}

async fn get_status(website: &str) -> Result<usize, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(website).send().await?;

    Ok(res.status().as_u16() as usize)
}

// Handle failure
fn handle_error(record: &mut Record, e: &reqwest::Error) {
    println!("Error fetching {}: {:?}", record.website, e);
    record.status = 0;
}
