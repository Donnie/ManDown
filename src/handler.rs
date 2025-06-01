use crate::alert::process;
use crate::data::{
    delete_user, delete_user_website, delete_website, get_user_by_telegram_id, get_websites_by_url,
    list_users_by_website,
};
use crate::http::get_status;
use crate::insert::put_user_website;
use crate::parse_url::{extract_hostname, read_url};
use crate::{data::list_websites_by_user, establish_connection};
use log::info;
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

pub async fn handle_list(bot: Bot, msg: Message) -> ResponseResult<()> {
    let mut conn = establish_connection();
    let telegram_id = msg.from().unwrap().id.0 as i32;
    let webs = list_websites_by_user(&mut conn, telegram_id)
        .await
        .expect("Error listing Websites");

    let websites = webs
        .iter()
        .map(|record| record.url.as_str())
        .collect::<Vec<&str>>()
        .join("\n");

    let output = format!("Here are your tracked domains:\n\n<pre>{}</pre>", websites);

    bot.send_message(msg.chat.id, output)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn handle_track(bot: Bot, msg: Message, website: String) -> ResponseResult<()> {
    let mut conn = establish_connection();
    let telegram_id = msg.from().unwrap().id.0 as i32;

    let (valid, normal, ssl) = read_url(&website);
    if !valid {
        bot.send_message(msg.chat.id, "Invalid URL!".to_string())
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let normal_status = get_status(&normal, &reqwest::Client::new()).await?;

    info!(
        "telegram_id: {} tried to track: {} and got status: {}",
        telegram_id, website, normal_status
    );

    let message = process(&normal, normal_status as i32);

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;
    if normal_status == 200 {
        put_user_website(&mut conn, &normal, telegram_id)
            .await
            .unwrap_or_else(|_| panic!("Error inserting site {}", &normal));
    }

    let ssl_status = get_status(&ssl, &reqwest::Client::new()).await?;
    let message = process(&ssl, ssl_status as i32);
    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .await?;
    if ssl_status == 200 {
        put_user_website(&mut conn, &ssl, telegram_id)
            .await
            .unwrap_or_else(|_| panic!("Error inserting site {}", &ssl));
    }

    Ok(())
}

pub async fn handle_untrack(bot: Bot, msg: Message, website: String) -> ResponseResult<()> {
    let mut conn = establish_connection();
    let telegram_id = msg.from().unwrap().id.0 as i32;
    let website = extract_hostname(&website);

    // get owner by telegram_id
    let user = get_user_by_telegram_id(&mut conn, telegram_id)
        .await
        .expect("Error getting user");

    // get sites matching url
    let sites = get_websites_by_url(&mut conn, &website)
        .await
        .expect("Error getting website");

    for site in sites {
        delete_user_website(&mut conn, site.id, user.id)
            .await
            .expect("Error deleting user website");

        // get list of owners from url
        let owners = list_users_by_website(&mut conn, site.id)
            .await
            .expect("Error listing owners");

        // if no owners then delete the website from the database
        let owners_count = owners.len();
        if owners_count == 0 {
            delete_website(&mut conn, site.id)
                .await
                .expect("Error deleting website");
        }
    }

    // get list of websites from telegram_id
    let sites = list_websites_by_user(&mut conn, telegram_id)
        .await
        .expect("Error listing websites");

    // if no sites then delete the owner from the database
    let sites_count = sites.len();
    if sites_count == 0 {
        delete_user(&mut conn, user.id)
            .await
            .expect("Error deleting user");
    }

    bot.send_message(
        msg.chat.id,
        format!("Website {} untracked successfully!", website),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
