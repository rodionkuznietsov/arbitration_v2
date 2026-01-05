use teloxide::prelude::*;
use teloxide::{Bot, types::Message};
use dotenv::dotenv;

pub async fn run() {
    dotenv().ok();
    
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            if text.to_lowercase() == "/start" {
                // bot.send_message(msg.chat.id, text).await?;
                bot.send_message(msg.chat.id, "Привет, нажми на Open, чтобы открыть бота").await?;
            }
        }
        Ok(())
    }).await;
}