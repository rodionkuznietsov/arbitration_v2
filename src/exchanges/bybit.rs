use std::{sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

use crate::exchanges::orderbook::{LocalOrderBook};

#[derive(Deserialize, Debug, Serialize)]
pub struct OrderbookResponse {
    pub topic: Option<String>,
    #[serde(rename = "data")]
    pub data: Option<Data>,
    #[serde(rename = "type")]
    pub order_type: Option<String>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Data {
    #[serde(rename = "symbol", alias="s")]
    pub symbol:  Option<String>,
    a: Option<Vec<Vec<String>>>,
    b: Option<Vec<Vec<String>>>,

    #[serde(rename = "lastPrice")]
    pub last_price: Option<String>
}

pub async fn connect(ticker: &str, channel_type: &str, local_ask_order_book: Arc<RwLock<LocalOrderBook>>) {
    
    let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Bybit] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Bybit-Websocket] is running");

    let order_book = format!("orderbook.50.{}USDT", ticker.to_uppercase());
    let price = format!("tickers.{}USDT", ticker.to_uppercase());
    
    write.send(Message::Text(
        serde_json::json!({
            "op": "subscribe",
            "channel_type": channel_type,
            "args": [
                order_book,
                price
            ]
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap().into_data();
        let data_string = String::from_utf8(data.to_vec()).unwrap();
        let json: OrderbookResponse = serde_json::from_str(&data_string).unwrap();
    
        fetch_data(json, local_ask_order_book.clone()).await;
    });

    read_future.await;
}

async fn fetch_data(data: OrderbookResponse, local_ask_order_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        let current = local_ask_order_book.read().await;
        current.clone()
    };
 
    if let (Some(topic), Some(d)) = (data.topic, data.data) {
        if topic.contains("tickers") {
            if let Some(last_price) = d.last_price {
                book.snapshot.last_price = last_price.parse::<f64>().expect("[Bybit] Failed to parse");
            }
        } else if topic.contains("orderbook") {
            if let Some(order_type) = data.order_type {
                // Snapshot processing
                if let Some(asks) = d.a {
                    for ask in asks {
                        let price = ask[0].parse::<f64>().unwrap();
                        let volume = ask[1].parse::<f64>().unwrap();

                        // println!("Ask Snapshot price: {price} volume: {volume}");
                        if order_type == "snapshot" {
                            book.snapshot.a.push((price, volume));
                        } 

                        // Удаляем запись
                        if volume == 0.0 {
                            book.snapshot.a.retain(|&a| a.0 != price);
                        } else {
                            // Ищем запись
                            if let Some(x) = book.snapshot.a.iter_mut().find(|x| x.0 == price) {
                                x.1 = volume;
                            } else {
                                // Добавляем запись
                                book.snapshot.a.push((price, volume));
                            }
                        }
                    }

                    if let Some(bids) = d.b {
                        for bid in bids {
                            let price = bid[0].parse::<f64>().unwrap();
                            let volume = bid[1].parse::<f64>().unwrap();

                            // println!("Bid Snapshot price: {price} volume: {volume}");
                            if order_type == "snapshot" {
                                book.snapshot.b.push((price, volume));
                            } 
                            
                            // Удаляем запись
                            if volume == 0.0 {
                                book.snapshot.b.retain(|&a| a.0 != price);
                            } else {
                                // Ищем запись
                                if let Some(x) = book.snapshot.b.iter_mut().find(|x| x.0 == price) {
                                    x.1 = volume;
                                } else {
                                    // Добавляем запись
                                    book.snapshot.b.push((price, volume));
                                }
                            }
                        }
                    } 
                } 
            }
        }
    }

    book.snapshot.a.sort_by(|a, b| b.0.total_cmp(&a.0));
    book.snapshot.b.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut book_lock = local_ask_order_book.write().await;
    *book_lock = book;
    
}