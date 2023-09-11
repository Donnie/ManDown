use teloxide::{prelude::*, types::ParseMode};
use crate::data::read_csv;

pub async fn handle_list(bot: Bot, msg: Message, filename: &str) -> ResponseResult<()> {
    let records = match read_csv(filename) {
        Ok(records) => records,
        Err(error) => {
            bot.send_message(msg.chat.id, format!("Failed to read records: {}", error)).await?;
            return Ok(());
        }
    };
    
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
