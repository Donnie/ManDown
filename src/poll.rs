use std::process::exit;

use crate::http::update_http_status;
use crate::data::{get_all_websites, compare_websites, write_all_websites};
use tokio::time;
use diesel::sqlite::SqliteConnection;

pub async fn check_urls(conn: &mut SqliteConnection, interval: u64) {
    loop {
        check_websites(conn).await;
        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

// Function to process DB
async fn check_websites(conn: &mut SqliteConnection) {
    // Read from DB
    let mut webs = get_all_websites(conn).expect("Error listing Websites");
    println!("Checking {} Websites...", webs.len());

    // Update HTTP status of each website
    update_http_status(&mut webs).await;

    let changed_webs = compare_websites(conn, webs).expect("Error comparing Websites");
    let web_count: usize = changed_webs.len();
    println!("Changed {} Websites.", web_count.clone());

    if web_count == 0 {
        println!("No websites changed, skipping database update");
        exit(0x0100);
    }

    // Write updated websites back to DB
    write_all_websites(conn, changed_webs).expect("Error updating Websites");
    println!("Updated all Websites.");
}
