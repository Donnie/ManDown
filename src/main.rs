mod poll;

mod handler;
use handler::{handle_about, handle_list, handle_track, handle_untrack};

mod alert;
mod baseline;
mod config;
mod data;
mod http;
mod insert;
mod mongo;
mod parse_url;
mod schema;

use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;

use mongodb::bson::Document;
use mongodb::{Client, options::ClientOptions};
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::poll::downtime_check;

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
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
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
        Command::List => handle_list(bot, msg, pool).await?,
        Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Track(website) => handle_track(bot, msg, website.to_lowercase(), pool).await?,
        Command::Untrack(website) => handle_untrack(bot, msg, website.to_lowercase(), pool).await?,
    };
    Ok(())
}

pub fn establish_connection() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Use the Tokio runtime for asynchronous execution
#[tokio::main]
async fn main() {
    // Load environment variables from a `.env` file if it exists
    dotenv().ok();

    let pool = establish_connection();

    // Initialize MongoDB client
    let uri = dotenvy::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client_options = ClientOptions::parse(uri)
        .await
        .expect("Failed to parse MongoDB URI");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");
    let db = client.database("mandown");
    let collection = Arc::new(db.collection::<Document>("websites"));

    // Run migrations using a connection from the pool
    let mut conn = pool.get().expect("Failed to get connection from pool");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply database migrations");

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
    tokio::spawn(async move {
        downtime_check(&collection_clone, interval, bot_clone).await;
    });

    // Start the bot's command loop
    let pool = pool.clone();
    Command::repl(bot, move |bot, msg, cmd| {
        let pool = pool.clone();
        async move { answer(bot, msg, cmd, pool).await }
    })
    .await;
}
