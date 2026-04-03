from telegram import Update, InlineKeyboardMarkup, InlineKeyboardButton, WebAppInfo
from telegram.ext import ApplicationBuilder, CommandHandler, ContextTypes

# Перенести ключ бота в переменные окружения для безопасности
BOT_TOKEN = "8354712003:AAEHnEqFC5Aj7bguPIi53aUI7L62yaD79T0"
app = ApplicationBuilder().token(BOT_TOKEN).build()

async def start(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    keyboard = InlineKeyboardMarkup([
        [InlineKeyboardButton(
            text="Open",
            web_app=WebAppInfo(url="https://arbitration-v2.vercel.app/")
        )]
    ])

    await update.message.reply_text("Открыть приложение:", reply_markup=keyboard)

app.add_handler(CommandHandler("start", start))