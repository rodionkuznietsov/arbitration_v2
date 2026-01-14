use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::exchanges::orderbook::{LocalOrderBook, OrderType, SnapshotUi};

mod exchanges;
mod exchange;
mod websocket;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ExchangeNames {
    ticker: String,
    long_exchange: String,
    short_exchange: String,
}

impl ExchangeNames {
    fn new() -> Self {
        Self { 
            ticker: String::new(),
            long_exchange: String::new(), 
            short_exchange: String::new() 
        }
    }
}

#[tokio::main]
async fn main() {
    let long_book = LocalOrderBook::new();
    let short_book = LocalOrderBook::new();

    let (sender_exchange_names, receiver_exchange_names) = mpsc::channel::<(String, String, String)>(200);
    let (sender_snapshot, _) = broadcast::channel::<(OrderType, SnapshotUi)>(1000);

    // Запускаем Websockets
    tokio::spawn({
        let long_book = long_book.clone();
        let short_book = short_book.clone();
        let snapshot_sender = sender_snapshot.clone();
        async move {
            exchange::run_websockets(
                long_book, 
                short_book, 
                receiver_exchange_names,
                snapshot_sender
            ).await;
        }
    });

    tokio::spawn({
        let long_book = long_book.clone();  
        let short_book = short_book.clone();  

        async move {
            websocket::connect_async(
                long_book, 
                short_book,
                sender_exchange_names,
                sender_snapshot
            ).await;
        } 
    });

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
