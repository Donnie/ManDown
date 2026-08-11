use crate::handler::{handle_about, handle_clear, handle_list, handle_track, handle_untrack};
use mongodb::{Collection, bson::Document};
use std::fmt::Debug;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "I can understand these commands"
)]
enum Command {
    #[command(description = "About ManDown")]
    About,
    #[command(description = "Clear your list of your followed domains")]
    Clear(String),
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

pub async fn start_command(
    bot: Bot,
    collection: Arc<Collection<Document>>,
    client: Arc<reqwest::Client>,
) {
    // Start the bot's command loop (long polling)
    Command::repl(bot, move |bot, msg, cmd| {
        let collection = collection.clone();
        let client = client.clone();
        async move { answer(bot, msg, cmd, collection, client).await }
    })
    .await;
}

pub async fn start_command_with_listener<L>(
    bot: Bot,
    collection: Arc<Collection<Document>>,
    client: Arc<reqwest::Client>,
    listener: L,
) where
    L: UpdateListener + Send,
    L::Err: Debug + Send,
{
    Command::repl_with_listener(
        bot,
        move |bot, msg, cmd| {
            let collection = collection.clone();
            let client = client.clone();
            async move { answer(bot, msg, cmd, collection, client).await }
        },
        listener,
    )
    .await;
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
        Command::Clear(confirmation) => handle_clear(bot, msg, &collection, confirmation).await?,
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
