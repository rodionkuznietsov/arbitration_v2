use std::{collections::{HashMap}, sync::Arc};

use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::exchanges::{self, orderbook::{BinanceOrderbookLocal, BybitOrderbookLocal}};

#[derive(Deserialize, Debug, Clone)]
pub struct WebsocketReceiverParams {
    pub exchanges: Exchanges,
    pub types: OrderTypes,
    pub ticker: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Exchanges {
    #[serde(rename="longExchange")]
    pub long_exchange: String,
    #[serde(rename="shortExchange")]
    pub short_exchange: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderTypes {
    #[serde(rename="longType")]
    pub long_type: String,
    #[serde(rename="shortType")]
    pub short_type: String
}

pub async fn connect_async() {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await.unwrap();
    
    println!("🌐 [Arbitration-Websocket] is running",);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, 
) {
    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    println!("🟢 [Arbitration-Websocket] Client connected");

    let binance_order_book = BinanceOrderbookLocal {
        snapshot: exchanges::orderbook::Snapshot 
        { 
            a: vec![], b: vec![],
            last_price: 0.0,
        }
    };

    let bybit_order_book = BybitOrderbookLocal {
        snapshot: exchanges::orderbook::Snapshot 
        { 
            a: vec![], b: vec![],
            last_price: 0.0,
        }
    };

    let binance_book = Arc::new(RwLock::new(binance_order_book));
    let binance_book_cl = binance_book.clone();

    let bybit_book = Arc::new(RwLock::new(bybit_order_book));
    let bybit_book_cl = bybit_book.clone();

    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    if let Ok(new_params) = serde_json::from_str::<WebsocketReceiverParams>(&msg.to_text().unwrap()) {
                        let new_params_cl = new_params.clone();
                        let binance_book = binance_book_cl.clone();
                        let bybit_book = bybit_book_cl.clone();

                        let long_exchange = new_params.exchanges.long_exchange.to_lowercase();
                        let short_exchange = new_params.exchanges.short_exchange.to_lowercase();
                        
                        if long_exchange == "binance" || short_exchange == "binance" {
                            tokio::spawn(async move {
                                exchanges::binance::connect(&new_params_cl.ticker.to_uppercase(), &new_params_cl.types.long_type, binance_book.clone()).await;
                            });
                        }

                        if short_exchange == "bybit" || long_exchange == "bybit" {
                            tokio::spawn(async move {
                                exchanges::bybit::connect(&new_params.ticker, &new_params.types.short_type, bybit_book.clone()).await;
                            });
                        }
                    }
                }
            }
        }
    });

    loop {
        let binance_order_book = {
            let book = binance_book.read().await;
            book.clone()
        };

        let bybit_order_book = {
            let book = bybit_book.read().await;
            book.clone()
        };

        let mut books = HashMap::new();
        books.insert("book1", serde_json::to_value(&binance_order_book).unwrap());
        books.insert("book2", serde_json::to_value(&bybit_order_book).unwrap());

        let json = serde_json::to_string(&books).unwrap();

        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            println!("Client disconnected");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}