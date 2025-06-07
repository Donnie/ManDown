use crate::alert::notify_user;
use crate::baseline::baseline_available;
use crate::data::{compare_websites, get_all_websites, write_all_websites};
use crate::http::{cust_client, update_http_statuses};
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use log::info;
use teloxide::Bot;
use tokio::time;

pub async fn check_urls(
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
    interval: u64,
    bot: Bot,
) {
    loop {
        let pool = pool.clone();
        let bot = bot.clone();
        tokio::spawn(async move {
            let mut conn = pool.get().unwrap();
            check_websites(&mut conn, bot).await;
        });

        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

// Function to process DB
async fn check_websites(conn: &mut SqliteConnection, bot: Bot) {
    info!("Checking Websites now");

    // Check baseline availability
    let result = baseline_available().await;
    if !result {
        return;
    }

    // Read from DB
    let mut webs = get_all_websites(conn).expect("Error listing Websites");
    if webs.is_empty() {
        return;
    }

    // Create client
    let client = cust_client(30);

    // Update HTTP status of each website
    update_http_statuses(&mut webs, &client).await;

    let changed_webs = compare_websites(conn, webs).expect("Error comparing Websites");
    let web_count: usize = changed_webs.len();

    info!("{} websites changed", web_count);

    if web_count == 0 {
        return;
    }

    // Notify user if websites changed
    notify_user(conn, bot, changed_webs.clone()).await;

    // Write updated websites back to DB
    write_all_websites(conn, changed_webs.clone()).expect("Error updating Websites");
}
