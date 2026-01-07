mod telegram;
use std::sync::{Arc};

// use telegram::bot;
use tokio::sync::RwLock;

mod exchanges;
mod websocket;

#[tokio::main]
async fn main() {

    let bybit_orderbook = exchanges::orderbook::OrderbookLocal { 
        snapshot: exchanges::orderbook::Snapshot 
        { 
            a: vec![], b: vec![] 
        }, 
        a: vec![], 
        b: vec![] 
    };

    let local_orderbook = Arc::new(RwLock::new(bybit_orderbook));

    tokio::spawn({
       let book = local_orderbook.clone();
       async move {
        websocket::connect_async(book).await;
       } 
    });

    // bot::run().await;
    exchanges::bybit::connect("btc", "spot", local_orderbook.clone()).await;
}
