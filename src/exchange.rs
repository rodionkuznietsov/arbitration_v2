use std::{sync::Arc, time::Duration};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast, mpsc::{self, Sender}};

use crate::{ExchangeNames, exchanges::{orderbook::{LocalOrderBook, OrderType, Snapshot, SnapshotUi}, *}};

#[derive(Debug)]
struct ExchangeWithParam {
    exchange_name: String, 
    book: Option<Arc<RwLock<LocalOrderBook>>>
}

impl ExchangeWithParam {
    pub async fn new(exchange_name: String, book: Option<Arc<RwLock<LocalOrderBook>>>) -> Self {
        Self { exchange_name, book }
    }

    pub async fn update_book(
        &self, 
        ticker: String, 
        snapshot_sender: broadcast::Sender<(OrderType, SnapshotUi)>,
        order_type: OrderType
    ) {
        if let Some(book) = &self.book {
            let book = book.read().await;            
            // Фильтруем по выброному токену пользователя
            if let Some(snaphsot) = book.books.get(&format!("{}usdt", ticker)) {
                println!("{:?}", snaphsot.last_price);
                let snapshot_ui = snaphsot.to_ui(6);
                snapshot_sender.send((order_type, snapshot_ui)).expect("[ExchangeWithParam] Failed to send snaphsot");
            }
        }
    }
}

pub async fn run_websockets(
    long_book: Arc<RwLock<LocalOrderBook>>,
    short_book: Arc<RwLock<LocalOrderBook>>,
    mut receiver: mpsc::Receiver<(String, String, String)>,
    snapshot_sender: broadcast::Sender<(OrderType, SnapshotUi)>
) {

    let binance_book = LocalOrderBook::new();
    let binance_book_cl = binance_book.clone();

    let bybit_book = LocalOrderBook::new();
    let bybit_book_cl = bybit_book.clone();

    // tokio::spawn(async move {
    //     binance_ws::connect(binance_book_cl).await;
    // });

    tokio::spawn(async move {
        bybit_ws::connect("spot", bybit_book_cl).await;
    });
    
    let exchanges = Arc::new(RwLock::new(ExchangeNames::new()));
    
    tokio::spawn({
        let le = exchanges.clone();
        async move {
            while let Some((t, l,s)) = receiver.recv().await {                
                let mut lock = le.write().await;
                *lock = ExchangeNames { 
                    ticker: t,
                    long_exchange: l, 
                    short_exchange: s 
                }
            }
        }
    });

    
    tokio::spawn(async move {
        loop {
            let exchanges = exchanges.read().await.clone();
            if exchanges.long_exchange != "" {
                let long_exchange = match exchanges.long_exchange.as_str() {
                    "binance" => ExchangeWithParam::new("binance".into(), Some(binance_book.clone())).await,
                    "bybit" => ExchangeWithParam::new("bybit".into(), Some(bybit_book.clone())).await,
                    _ => continue,
                };  
                long_exchange.update_book(exchanges.ticker.clone(), snapshot_sender.clone(), OrderType::Long).await;   
            }

            if exchanges.short_exchange != "" {
                let short_exchange = match exchanges.short_exchange.as_str() {
                    "binance" => ExchangeWithParam::new("binance".into(), Some(binance_book.clone())).await,
                    "bybit" => ExchangeWithParam::new("bybit".into(), Some(bybit_book.clone())).await,
                    _ => continue,
                };  
                short_exchange.update_book(exchanges.ticker.clone(), snapshot_sender.clone(), OrderType::Short).await;   
            }
        }
    });
}