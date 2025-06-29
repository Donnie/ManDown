use crate::alert::process;
use crate::format::format_website_list;
use crate::http::HttpClient;
use crate::mongo::{clear_user_websites, delete_sites_by_hostname, get_user_websites, put_site};
use crate::parse_url::{extract_hostname, read_url};
use futures::join;
use mongodb::Collection;
use mongodb::bson::Document;
use std::sync::Arc;
use teloxide::{prelude::*, types::ParseMode};

pub async fn handle_about(bot: Bot, msg: Message) -> ResponseResult<()> {
    let output = "<b>ManDown</b>:
Open Source on <a href='https://github.com/Donnie/ManDown'>GitHub</a>
Hosted on GCP in us-east-1
No personally identifiable information is stored or used by this bot.";

    bot.send_message(msg.chat.id, output)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn handle_clear(
    bot: Bot,
    msg: Message,
    collection: &Collection<Document>,
    confirmation: String,
) -> ResponseResult<()> {
    let mut message = r#"
To clear your entire list of followed domains, please type:
<pre>
/clear confirmed
</pre>
"#
    .to_string();

    if confirmation.to_lowercase() == "confirmed" {
        let telegram_id = msg.from().unwrap().id.0 as i32;
        message = match clear_user_websites(collection, telegram_id).await {
            Ok(count) => format!("Successfully cleared {count} site(s)"),
            Err(e) => {
                log::error!("Failed to clear user websites: {e}");
                format!("Failed to clear user websites: {e}")
            }
        };
    }
    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn handle_list(
    bot: Bot,
    msg: Message,
    collection: &Collection<Document>,
) -> ResponseResult<()> {
    let telegram_id = msg.from().unwrap().id.0 as i32;

    let message = match get_user_websites(collection, telegram_id).await {
        Ok(websites) => format_website_list(&websites),
        Err(e) => {
            log::error!("Failed to get user websites: {e}");
            "Failed to get user websites".to_string()
        }
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn check_and_track_url(
    url: &str,
    collection: &Collection<Document>,
    telegram_id: i32,
    client: &reqwest::Client,
) -> String {
    let status = client.get_status_code(url).await;
    let mut message = process(url, status as i32);

    if status == 200 {
        if let Err(e) = put_site(collection, url, telegram_id).await {
            log::error!("Failed to insert site {url}: {e}");
            message = format!("Failed to track <code>{url}</code>");
        }
    }

    message
}

pub async fn handle_track(
    bot: Bot,
    msg: Message,
    website: String,
    collection: &Collection<Document>,
    client: Arc<reqwest::Client>,
) -> ResponseResult<()> {
    let telegram_id = msg.from().unwrap().id.0 as i32;

    let (valid, normal, ssl) = read_url(&website);
    if !valid {
        bot.send_message(msg.chat.id, "Invalid URL!".to_string())
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let normal_check = check_and_track_url(&normal, collection, telegram_id, &client);
    let ssl_check = check_and_track_url(&ssl, collection, telegram_id, &client);

    let (normal_result, ssl_result) = join!(normal_check, ssl_check);

    let messages = [normal_result, ssl_result];

    if !messages.is_empty() {
        bot.send_message(msg.chat.id, messages.join("\n\n"))
            .parse_mode(ParseMode::Html)
            .await?;
    }

    Ok(())
}

pub async fn handle_untrack(
    bot: Bot,
    msg: Message,
    website: String,
    collection: &Collection<Document>,
) -> ResponseResult<()> {
    let telegram_id = msg.from().unwrap().id.0 as i32;
    let hostname = extract_hostname(&website);
    if hostname.len() < 3 {
        bot.send_message(msg.chat.id, "Invalid URL!".to_string())
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let result = delete_sites_by_hostname(collection, &hostname, telegram_id).await;

    let message = match result {
        Ok(0) => format!("No sites found for {hostname}"),
        Ok(count) => format!("Successfully untracked {count} site(s) for {hostname}"),
        Err(e) => {
            log::error!("Error untracking {hostname}: {e}");
            format!("An error occurred while untracking {hostname}")
        }
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
