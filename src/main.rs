mod http;
use http::update_http_status;

mod data;
use data::read_csv;
use data::write_csv;

use csv::Error;
use std::path::Path;
use tokio::time;
use dotenv::dotenv;

// Function to process CSV
async fn check_records(filename: &str) -> Result<(), Error> {
    // Read from CSV
    let mut records = read_csv(filename)?;

    // Update HTTP status of each website
    update_http_status(&mut records).await;

    // Write updated records back to CSV
    write_csv(filename, records)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let filename = dotenv::var("DBFILE").unwrap_or("db/db.csv".to_string());
    let interval: u64 = dotenv::var("FREQ")
        .unwrap_or("600".to_string())
        .parse()
        .expect("Interval must be a number");

    // Check that the file exists
    if !Path::new(&filename).exists() {
        println!("The file {} does not exist", filename);
        return;
    }

    loop {
        match check_records(&filename).await {
            Ok(_) => println!("CSV file updated successfully!"), 
            Err(e) => println!("Error processing CSV file: {:?}", e),
        }

        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}
