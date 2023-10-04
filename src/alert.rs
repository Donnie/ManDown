use diesel::SqliteConnection;
use teloxide::{prelude::*, types::ParseMode};

use crate::{data::list_users_by_website, schema::Website};

pub async fn notify_user(conn: &mut SqliteConnection, bot: Bot, changed_webs: Vec<Website>) {
    for website in changed_webs {
        // Look up users for this website
        let users = list_users_by_website(conn, website.id)
        .await
        .expect("Error getting Users");
        
        // Notify each user
        for user in users {
            let chat_id = ChatId(user.telegram_id as i64);
            
            let message = process(&website.url, website.status);
            
            bot.send_message(chat_id, message)
                .parse_mode(ParseMode::Html)
                .await
                .expect(&format!("Error sending messages to {}", chat_id));
        }
    }
}

fn process(site: &str, code: i32) -> String {
    let mut output = format!("Site: {}\n\n", site);
    
    match code {
        0 | 1 => output += &format!("Hoppla! We faced an error trying to reach the site! 🤒"),
        200..=299 => output += &format!("Joohoo! It's live and kicking! 🙂\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>", code, code),
        400..=499 => output += &format!("Erm! Did I do something wrong? 🤔\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>", code, code),
        500..=599 => output += &format!("Schade! It's down or inaccessible to me! 😟\n\nStatus: <a href='https://httpstatuses.com/{}'>{}</a>", code, code),
        _ => output += "Something is fishy! 🐟",
    }
    
    output
}
