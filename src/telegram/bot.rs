use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};
use teloxide::{Bot, types::Message};
use dotenv::dotenv;

pub async fn run() {
    dotenv().ok();
    
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            if text.to_lowercase() == "/start" {
                // bot.send_message(msg.chat.id, text).await?;
                show_keyboard(&bot, msg.chat.id).await.expect("[TelegramBot] Failed to show keybaord");
            }
        }
        Ok(())
    }).await;
}

async fn show_keyboard(bot: &Bot, chat_id: ChatId) -> anyhow::Result<()> {
    let keyboard = KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Open")]
    ]);

    bot.send_message(chat_id, "WoW")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}