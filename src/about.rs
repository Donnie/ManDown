use teloxide::prelude::*;

pub async fn handle_about(bot: Bot, msg: Message) -> ResponseResult<()> {
  let output = "*ManDown*:
  Open Source on [GitHub](https://github.com/Donnie/ManDown)
  Hosted on Vultr.com in New Jersey, USA
  No personally identifiable information is stored or used by this bot.";

  bot.send_message(msg.chat.id, output).await?;

  Ok(())
}
