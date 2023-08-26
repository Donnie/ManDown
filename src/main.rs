mod http;
use http::update_http_status;

mod data;
use data::read_csv;
use data::write_csv;

use csv::Error;
use std::path::Path;

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
    let filename = "db/db.csv";

    // Check that the file exists
    if !Path::new(filename).exists() {
        println!("The file {} does not exist", filename);
        return;
    }

    // Process the CSV file
    match check_records(filename).await {
        Ok(_) => println!("CSV file updated successfully!"),
        Err(e) => println!("Error processing CSV file: {:?}", e),
    }
}
