use dotenvy::dotenv;
use man_down_core::command::start_command_with_listener;
use man_down_core::config::init_logger;
use man_down_core::http::cust_client;
use man_down_core::mongo::init_mongo;
use teloxide::prelude::*;
use teloxide::update_listeners::webhooks;
use url::Url;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logger();

    let collection = init_mongo().await;
    let http_client = cust_client(30);
    let bot = Bot::from_env();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = ([0, 0, 0, 0], port).into();
    let url: Url = std::env::var("WEBHOOK_URL")
        .expect("WEBHOOK_URL must be set")
        .parse()
        .expect("WEBHOOK_URL must be a valid URL");
    let webhook_token = std::env::var("WEBHOOK_TOKEN").expect("WEBHOOK_TOKEN must be set");

    log::info!("Webhook bot listening on 0.0.0.0:{port}");

    let options = webhooks::Options::new(addr, url).secret_token(webhook_token);
    let listener = webhooks::axum(bot.clone(), options)
        .await
        .expect("Couldn't setup webhook");

    start_command_with_listener(bot, collection, http_client, listener).await;
}
