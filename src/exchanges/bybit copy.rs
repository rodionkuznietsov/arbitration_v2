use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{io::AsyncWriteExt, sync::RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

#[derive(Deserialize, Debug, Serialize)]
pub struct Orderbook {
    #[serde(rename = "data")]
    pub data: Option<Data>,
    #[serde(rename = "type")]
    pub order_type: Option<String>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Data {
    #[serde(rename = "s")]
    pub symbol:  Option<String>,
    a: Option<Vec<Vec<String>>>,
    b: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderbookLocal {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
}

pub async fn connect(ticker: &str, channel_type: &str, local_ask_order_book: Arc<RwLock<OrderbookLocal>>) {
    
    let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Bybit] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Bybit-Websocket] is running");

    let ticker_target = format!("orderbook.1.{}USDT", ticker.to_uppercase());
    
    write.send(Message::Text(
        serde_json::json!({
            "op": "subscribe",
            "channel_type": channel_type,
            "args": [ticker_target]
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap().into_data();
        let data_string = String::from_utf8(data.to_vec()).unwrap();
        let json: Orderbook = serde_json::from_str(&data_string).unwrap();
    
        parsing_the_data(json, local_ask_order_book.clone()).await;
    });

    read_future.await;
}

async fn parsing_the_data(data: Orderbook, local_ask_order_book: Arc<RwLock<OrderbookLocal>>) {
    let mut book = {
        let current = local_ask_order_book.read().await;
        current.clone()
    };
    
    if let Some(d) = data.data {
        let asks = d.a;
        if let Some(ask) = asks {
            if ask.len() > 0 {
                for a in ask {
                    let price = a[0].parse::<f64>().unwrap();
                    let volume = a[1].parse::<f64>().unwrap();

                    if volume == 0.0 {
                        book.a.retain(|(local_price, _)| *local_price != price);
                    } else if volume > 0.1 {
                        println!("Ask: {}; {}", price, volume);

                        if let Some(l) = book.a.iter_mut().find(|(p, _)| *p == price) {
                            l.1 = volume
                        } else {
                            book.a.push((price, volume));
                        }
                    }
                }
            }
        }

        let bids = d.b;
        if let Some(bid) = bids {
            if bid.len() > 0 {
                for b in bid {
                    let price = b[0].parse::<f64>().unwrap();
                    let volume = b[1].parse::<f64>().unwrap();

                    if volume == 0.0 {
                        book.b.retain(|(local_price, _)| *local_price != price);
                        
                    } else {
                        println!("Bid: {}; {}", price, volume);

                        if let Some(l) = book.b.iter_mut().find(|(p, _)| *p == price) {
                            l.1 = volume
                        } else {
                            book.b.push((price, volume));
                        }
                    }
                }
            }
        }
    }

    book.a.sort_by(|a, b| b.0.total_cmp(&a.0));
    book.b.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut book_lock = local_ask_order_book.write().await;
    *book_lock = book;
    
}