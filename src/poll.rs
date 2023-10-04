use crate::http::update_http_status;
use crate::alert::notify_user;
use crate::data::{get_all_websites, compare_websites, write_all_websites};
use teloxide::Bot;
use tokio::time;
use diesel::sqlite::SqliteConnection;

pub async fn check_urls(conn: &mut SqliteConnection, interval: u64, bot: Bot) {
    loop {
        check_websites(conn, bot.clone()).await;
        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

// Function to process DB
async fn check_websites(conn: &mut SqliteConnection, bot: Bot) {
    // Read from DB
    let mut webs = get_all_websites(conn).expect("Error listing Websites");
    println!("Checking {} Websites...", webs.len());

    // Update HTTP status of each website
    update_http_status(&mut webs).await;

    let changed_webs = compare_websites(conn, webs).expect("Error comparing Websites");
    let web_count: usize = changed_webs.len();
    println!("Changed {} Websites.", web_count.clone());

    if web_count.clone() == 0 {
        println!("No websites changed, skipping database update");
        return
    }

    // Notify user if websites changed
    notify_user(conn, bot, changed_webs.clone()).await;

    // Write updated websites back to DB
    write_all_websites(conn, changed_webs.clone()).expect("Error updating Websites");
    println!("Updated {} Websites.", web_count.clone());
}
