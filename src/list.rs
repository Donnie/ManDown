use teloxide::{prelude::*, types::ParseMode};
use crate::{data::list_websites_by_user, establish_connection};

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
