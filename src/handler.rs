use teloxide::{prelude::*, types::ParseMode};
use crate::data::read_url;
use crate::{data::list_websites_by_user, establish_connection};
use crate::http::get_status;
use crate::alert::process;

pub async fn handle_about(bot: Bot, msg: Message) -> ResponseResult<()> {
    let output = "<b>ManDown</b>:
  Open Source on <a href='https://github.com/Donnie/ManDown'>GitHub</a>
  Hosted on GCP in us-east-1
  No personally identifiable information is stored or used by this bot.";
  
    bot.send_message(msg.chat.id, output).parse_mode(ParseMode::Html).await?;
  
    Ok(())
  }
  

pub async fn handle_list(bot: Bot, msg: Message) -> ResponseResult<()> {
    let mut conn = establish_connection();
    let telegram_id = msg.from().unwrap().id.0 as i32;
    let webs = list_websites_by_user(&mut conn, telegram_id).await
        .expect("Error listing Websites");
    
    let websites = webs
        .iter()
        .map(|record| record.url.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    
    let output = format!("Here are your tracked domains:\n\n<pre>{}</pre>", websites);

    bot.send_message(msg.chat.id, output).parse_mode(ParseMode::Html).await?;
    
    Ok(())
}

pub async fn handle_track(bot: Bot, msg: Message, website: String) -> ResponseResult<()> {
    // let mut conn = establish_connection();

    let (valid, normal, ssl) = read_url(&website);
    if !valid {
        bot.send_message(msg.chat.id, format!("Invalid URL!")).parse_mode(ParseMode::Html).await?;
        return Ok(());
    }

    let normal_status = get_status(&normal).await?;
    let message = process(&normal, normal_status as i32);
    bot.send_message(msg.chat.id, message).parse_mode(ParseMode::Html).await?;

    let ssl_status = get_status(&ssl).await?;
    let message = process(&ssl, ssl_status as i32);
    bot.send_message(msg.chat.id, message).parse_mode(ParseMode::Html).await?;

    Ok(())
}
