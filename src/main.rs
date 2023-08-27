mod poll;
use poll::run_poll;

mod about;
use about::handle_about;

mod data;
mod http;

use std::path::Path;
use dotenv::dotenv;

use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::repls::CommandReplExt;

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

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
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
        Command::List => {
            bot.send_message(
            msg.chat.id,
            Command::descriptions().to_string(),
        ).await?;
        }
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let filename = dotenv::var("DBFILE").unwrap_or("db/db.csv".to_string());
    let interval: u64 = dotenv::var("FREQ")
        .unwrap_or("600".to_string())
        .parse()
        .expect("FREQ must be a number");

    // Check that the file exists
    if !Path::new(&filename).exists() {
        panic!("The DBFILE {} does not exist", filename);
    }

    // Start polling function
    tokio::spawn(async move {
        run_poll(filename, interval).await;
    });

    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}
