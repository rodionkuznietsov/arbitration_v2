mod telegram;
use std::{time::Duration};

mod exchanges;
mod websocket;

#[tokio::main]
async fn main() {

    tokio::spawn({
        async move {
            websocket::connect_async().await;
        } 
    });

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // bot::run().await;
    // exchanges::bybit::connect("btc", "spot", local_orderbook.clone()).await;
    // exchanges::binance::connect("btc", "spot", binance_local_book.clone()).await;
}
