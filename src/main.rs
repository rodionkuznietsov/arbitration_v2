mod telegram;
use std::sync::{Arc};

use telegram::bot;
use tokio::sync::RwLock;

use crate::exchanges::bybit::OrderbookLocal;

mod exchanges;
mod websocket;

#[tokio::main]
async fn main() {

    let local_orderbook = Arc::new(RwLock::new(OrderbookLocal { a: vec![], b: vec![] }));

    tokio::spawn({
       let book = local_orderbook.clone();
       async move {
        websocket::connect_async(book).await;
       } 
    });

    // bot::run().await;
    exchanges::bybit::connect("btc", "spot", local_orderbook.clone()).await;
}
