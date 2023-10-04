use teloxide::{prelude::*, types::ParseMode};
use crate::data::get_all_websites;
use diesel::sqlite::SqliteConnection;

pub async fn handle_list(bot: Bot, msg: Message, conn: &mut SqliteConnection) -> ResponseResult<()> {
    let records = get_all_websites(conn);
    
    let user_id = msg.from().unwrap().id.0 as usize;
    
    let websites = records
        .iter()
        .filter(|r| r.user == user_id)
        .map(|record| record.website.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    
    let output = format!("Here are your tracked domains:\n\n<pre>{}</pre>", websites);
    
    bot.send_message(msg.chat.id, output).parse_mode(ParseMode::Html).await?;
    
    Ok(())
}
