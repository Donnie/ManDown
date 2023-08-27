mod poll;
use poll::checkup;

mod about;
use about::handle_about;

mod list;
use list::handle_list;

mod data;
mod http;

use dotenv::dotenv;
use std::path::Path;
use std::sync::Arc;

use teloxide::{
    prelude::*,
    repls::CommandReplExt,
    utils::command::BotCommands,
};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "I can understand these commands")]
enum Command {
    #[command(description = "About ManDown")]
    About,
    #[command(description = "handle a website.")]
    Clear,
    #[command(description = "I am here to help!")]
    Help,
    #[command(description = "handle a website.")]
    List,
    #[command(description = "I am here to help!")]
    Start,
    #[command(description = "handle a website.")]
    Track(String),
    #[command(description = "handle a website.")]
    Untrack(String),
}

async fn answer(bot: Bot, msg: Message, cmd: Command, filename: Arc<String>) -> ResponseResult<()> {
    match cmd {
        Command::About => handle_about(bot, msg).await?,
        Command::Clear => {
            bot.send_message(
                msg.chat.id,
                Command::descriptions().to_string(),
            ).await?;
        }
        Command::Help => {
            bot.send_message(
            msg.chat.id,
            Command::descriptions().to_string(),
        ).await?;
        }
        Command::List => handle_list(bot, msg, &*filename).await?,
        Command::Start => {
            bot.send_message(
            msg.chat.id,
            Command::descriptions().to_string(),
        ).await?;
        }
        Command::Track(website) => {
            bot.send_message(
                msg.chat.id,
                format!("{website}"),
            ).await?;
        }
        Command::Untrack(website) => {
            bot.send_message(
                msg.chat.id,
                format!("{website}"),
            ).await?;
        }
    };
    Ok(())
}

// Use the Tokio runtime for asynchronous execution
#[tokio::main]
async fn main() {
    // Load environment variables from a `.env` file if it exists
    dotenv().ok();

    // Get the database filename from the environment variable or use a default value
    let filename = Arc::new(dotenv::var("DBFILE").unwrap_or("db/db.csv".to_string()));

    // Get the polling frequency from the environment variable or use a default value
    let interval: u64 = dotenv::var("FREQ")
        .unwrap_or("600".to_string())
        .parse()
        .expect("FREQ must be a number");

    // Clone the filename for use in the asynchronous block
    let dbfile = filename.clone();

    // Ensure the database file exists before proceeding
    if !Path::new(&*filename).exists() {
        panic!("The DBFILE {} does not exist", filename);
    }

    // Start the polling function in the background
    tokio::spawn(async move {
        checkup(dbfile.to_string(), interval).await;
    });

    // Initialize the bot from environment variables
    let bot = Bot::from_env();

    // Start the bot's command loop
    Command::repl(bot, move |bot, msg, cmd| answer(bot, msg, cmd, filename.clone())).await;
}
