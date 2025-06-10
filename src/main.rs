mod alert;
mod baseline;
mod command;
mod config;
mod format;
mod handler;
mod http;
mod mongo;
mod parse_url;
mod poll;

use command::start_command;
use config::init_logger;
use dotenvy::dotenv;
use http::cust_client;
use mongo::init_mongo;
use poll::start_downtime_checker;
use teloxide::prelude::*;

// Use the Tokio runtime for asynchronous execution
#[tokio::main]
async fn main() {
    // Load environment variables from a `.env` file if it exists
    dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    log::info!("Bot started");

    start_downtime_checker(bot.clone(), collection.clone(), http_client.clone());

    start_command(bot.clone(), collection.clone(), http_client.clone()).await;
}
