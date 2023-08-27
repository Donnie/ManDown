use crate::http::update_http_status;
use crate::data::read_csv;
use crate::data::write_csv;

use csv::Error;
use tokio::time;

pub async fn run_poll(filename: String, interval: u64) {
    loop {
        match check_records(&filename).await {
            Ok(_) => println!("CSV file updated successfully!"), 
            Err(e) => println!("Error processing CSV file: {:?}", e),
        }
        
        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

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
