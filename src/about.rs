use teloxide::{prelude::*, types::ParseMode};

pub async fn handle_about(bot: Bot, msg: Message) -> ResponseResult<()> {
  let output = "<b>ManDown</b>:
Open Source on <a href='https://github.com/Donnie/ManDown'>GitHub</a>
Hosted on Vultr.com in New Jersey, USA
No personally identifiable information is stored or used by this bot.";

  bot.send_message(msg.chat.id, output).parse_mode(ParseMode::Html).await?;

  Ok(())
}
