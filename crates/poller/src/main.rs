use dotenvy::dotenv;
use man_down_core::config::init_logger;
use man_down_core::http::cust_client;
use man_down_core::mongo::init_mongo;
use man_down_core::poll::run_once;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    run_once(&collection, bot, http_client).await;
}
