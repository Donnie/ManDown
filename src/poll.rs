use crate::schema;
use tokio::time;
use diesel::{prelude::*, sqlite::SqliteConnection};

pub async fn check_urls(conn: &mut SqliteConnection, interval: u64) {
    loop {
        let urls = get_all_urls(conn).expect("Error fetching URLs");
        println!("Checking {} URLs...", urls.len());
        
        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}

fn get_all_urls(conn: &mut SqliteConnection) -> Result<Vec<String>, diesel::result::Error> {
    use schema::websites::dsl as websites_dsl;

    // Fetch all the URLs from the websites table
    let urls: Vec<String> = websites_dsl::websites
        .select(websites_dsl::url)
        .load(conn)?;

    Ok(urls)
}
