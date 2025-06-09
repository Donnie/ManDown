use log::info;
use teloxide::{prelude::*, types::ParseMode};

use crate::mongo::Website;

pub async fn alert_users(bot: Bot, changed_webs: &[Website]) {
    for website in changed_webs {
        let tele_id = website.telegram_id.parse::<i64>().unwrap();
        let chat_id = ChatId(tele_id);

        let message = process(&website.url, website.status);

        if let Err(e) = bot
            .send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .await
        {
            info!("Failed to send message to {}: {}", chat_id, e);
        }
    }
}

pub fn process(site: &str, code: i32) -> String {
    let mut output = format!("Site: {}\n\n", site);

    match code {
        0 | 1 => output += "Hoppla! We faced an error trying to reach the site! 🤒",
        200..=299 => {
            output += &format!(
                "Joohoo! It's live and kicking! 🙂\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>",
                code, code
            )
        }
        400..=499 => {
            output += &format!(
                "Erm! Did I do something wrong? 🤔\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>",
                code, code
            )
        }
        500..=599 => {
            output += &format!(
                "Schade! It's down or inaccessible to me! 😟\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>",
                code, code
            )
        }
        _ => output += "Something is fishy! 🐟",
    }

    output
}
