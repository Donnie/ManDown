mod poll;
use poll::run_poll;

mod data;
mod http;

use std::path::Path;
use dotenv::dotenv;

use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::repls::CommandReplExt;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "I am here to help!")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(
            msg.chat.id,
            Command::descriptions().to_string(),
        ).await?,
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
