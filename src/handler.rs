use crate::alert::process;
use crate::http::{HttpClient, cust_client};
use crate::mongo::{delete_sites_by_hostname, put_site};
use crate::parse_url::{extract_hostname, read_url};
use futures::{StreamExt, join};
use mongodb::Collection;
use mongodb::bson::{Document, doc};
use teloxide::RequestError;
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

pub async fn handle_list(
    bot: Bot,
    msg: Message,
    collection: &Collection<Document>,
) -> ResponseResult<()> {
    let telegram_id = msg.from().unwrap().id.0 as i32;

    // Create a filter to match documents with the user's telegram_id
    let filter = doc! { "telegram_id": format!("{}", telegram_id) };

    // Find documents matching the filter
    let mut cursor = collection.find(filter).await.map_err(|e| {
        log::error!("Failed to query MongoDB: {}", e);
        RequestError::from(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;

    // Collect all website URLs into a vector
    let mut websites: Vec<String> = Vec::new();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(doc) => {
                if let Ok(url) = doc.get_str("url") {
                    websites.push(url.to_string());
                }
            }
            Err(e) => {
                log::error!("Failed to read document: {}", e);
                return Err(RequestError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e,
                )));
            }
        }
    }

    websites.sort();

    // Create a formatted list of websites
    let websites_list = websites.join("\n");
    let output = format!(
        "Here are your tracked domains:\n\n<pre>{}</pre>",
        websites_list
    );

    bot.send_message(msg.chat.id, output)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

async fn check_and_track_url(
    url: &str,
    collection: &Collection<Document>,
    telegram_id: i32,
    client: &reqwest::Client,
) -> ResponseResult<Option<String>> {
    let status = client.get_status_code(url).await;
    let message = process(url, status as i32);

    if status == 200 {
        put_site(collection, url, telegram_id)
            .await
            .unwrap_or_else(|_| panic!("Error inserting site {}", url));
    }

    Ok(Some(message))
}

pub async fn handle_track(
    bot: Bot,
    msg: Message,
    website: String,
    collection: &Collection<Document>,
) -> ResponseResult<()> {
    let telegram_id = msg.from().unwrap().id.0 as i32;

    let (valid, normal, ssl) = read_url(&website);
    if !valid {
        bot.send_message(msg.chat.id, "Invalid URL!".to_string())
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let client = cust_client(30);
    let normal_check = check_and_track_url(&normal, collection, telegram_id, &client);
    let ssl_check = check_and_track_url(&ssl, collection, telegram_id, &client);

    let (normal_result, ssl_result) = join!(normal_check, ssl_check);

    let messages: Vec<String> = [normal_result?, ssl_result?]
        .into_iter()
        .flatten()
        .collect();

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
        Ok(0) => format!("No sites found for {}", hostname),
        Ok(count) => format!("Successfully untracked {} site(s) for {}", count, hostname),
        Err(e) => {
            log::error!("Error untracking {}: {}", hostname, e);
            format!("An error occurred while untracking {}", hostname)
        }
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
