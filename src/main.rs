mod alert;
mod baseline;
mod config;
mod handler;
mod http;
mod mongo;
mod parse_url;
mod poll;

use dotenvy::dotenv;
use handler::{handle_about, handle_list, handle_track, handle_untrack};
use http::cust_client;
use mongodb::{Client, Collection, bson::Document, options::ClientOptions};
use poll::downtime_check;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "I can understand these commands"
)]
enum Command {
    #[command(description = "About ManDown")]
    About,
    #[command(description = "Clear your list of your followed domains")]
    Clear,
    #[command(description = "I am here to help!")]
    Help,
    #[command(description = "Get a list of your followed domains")]
    List,
    #[command(description = "I am here to help!")]
    Start,
    #[command(description = "Add to the list of tracked websites")]
    Track(String),
    #[command(description = "Remove from the list of tracked websites")]
    Untrack(String),
}

async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    collection: Arc<Collection<Document>>,
    client: Arc<reqwest::Client>,
) -> ResponseResult<()> {
    match cmd {
        Command::About => handle_about(bot, msg).await?,
        Command::Clear => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::List => handle_list(bot, msg, &collection).await?,
        Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Track(website) => {
            handle_track(bot, msg, website.to_lowercase(), &collection, client).await?
        }
        Command::Untrack(website) => {
            handle_untrack(bot, msg, website.to_lowercase(), &collection).await?
        }
    };
    Ok(())
}

// Use the Tokio runtime for asynchronous execution
#[tokio::main]
async fn main() {
    // Load environment variables from a `.env` file if it exists
    dotenv().ok();

    // Initialize MongoDB client
    let uri = dotenvy::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client_options = ClientOptions::parse(uri)
        .await
        .expect("Failed to parse MongoDB URI");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");
    let db = client.database("mandown");
    let collection = Arc::new(db.collection::<Document>("websites"));
    let http_client = Arc::new(cust_client(30));

    // Get the polling frequency from the environment variable or use a default value
    let interval: u64 = dotenvy::var("FREQ")
        .unwrap_or("600".to_string())
        .parse()
        .expect("FREQ must be a number");

    // Initialize the bot from environment variables
    let bot = Bot::from_env();

    // Start the polling function in the background
    let bot_clone = bot.clone();
    let collection_clone = collection.clone();
    let http_client_clone = http_client.clone();
    tokio::spawn(async move {
        downtime_check(&collection_clone, interval, bot_clone, http_client_clone).await;
    });

    // Start the bot's command loop
    Command::repl(bot, move |bot, msg, cmd| {
        let collection = collection.clone();
        let http_client = http_client.clone();
        async move { answer(bot, msg, cmd, collection, http_client).await }
    })
    .await;
}
