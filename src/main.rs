mod telegram;
use telegram::bot;

#[tokio::main]
async fn main() {
    bot::run().await;
}
