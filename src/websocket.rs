use std::{collections::{HashMap}, sync::Arc};

use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::{RwLock}};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::exchanges::{self, orderbook::{LocalOrderBook}};

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

enum Exchange {
    Bybit,
    Binance
}

impl Exchange {
    async fn connect(&self, ticker: &str, order_type: &str, book: Arc<RwLock<LocalOrderBook>>) {
        match self {
            Exchange::Binance => {
                exchanges::binance::connect(ticker, order_type, book.clone()).await;
            }
            Exchange::Bybit => {
                exchanges::bybit::connect(ticker, order_type, book.clone()).await;
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, 
) {
    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    println!("🟢 [Arbitration-Websocket] Client connected");

    let long_book = LocalOrderBook::new();
    let long_book_cl = long_book.clone();

    let short_book = LocalOrderBook::new();
    let short_book_cl = short_book.clone();

    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    if let Ok(new_params) = serde_json::from_str::<WebsocketReceiverParams>(&msg.to_text().unwrap()) {
                        let long_exchange = new_params.exchanges.long_exchange.to_lowercase();
                        let short_exchange = new_params.exchanges.short_exchange.to_lowercase();

                        let long_exhange_connect = match long_exchange.as_str() {
                            "binance" => Exchange::Binance,
                            "bybit" => Exchange::Bybit,
                            _ => panic!("unknown exchange")
                        };

                        let long_ticker = new_params.ticker.clone();
                        let long_book = long_book_cl.clone();
                        let long_order_type = new_params.types.long_type;

                        tokio::spawn(async move {
                            long_exhange_connect.connect(long_ticker.as_str(), long_order_type.as_str(), long_book.clone()).await;
                        });

                        let short_exhange_connect = match short_exchange.as_str() {
                            "binance" => Exchange::Binance,
                            "bybit" => Exchange::Bybit,
                            _ => panic!("unknown exchange")
                        };
                        
                        let short_ticker = new_params.ticker.clone();
                        let short_book = short_book_cl.clone();
                        let short_order_type = new_params.types.short_type;

                        tokio::spawn(async move {
                            short_exhange_connect.connect(short_ticker.as_str(), short_order_type.as_str(), short_book.clone()).await;
                        });
                    }
                }
            }
        }
    });
    
    loop {
        let long_book = long_book.read().await;
        let short_book = short_book.read().await;
        
        let mut books = HashMap::new();
        books.insert("book1", long_book.clone());
        books.insert("book2", short_book.clone());

        let json = serde_json::to_string(&books).unwrap();

        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            println!("Client disconnected");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
}