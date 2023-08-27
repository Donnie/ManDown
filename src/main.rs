mod poll;
use poll::run_poll;

mod data;
mod http;

use std::path::Path;
use dotenv::dotenv;

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

    run_poll(filename, interval).await;
}
