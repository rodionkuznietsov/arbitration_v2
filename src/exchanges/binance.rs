use std::{sync::Arc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::exchanges::orderbook::{LocalOrderBook};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SnapshotResponse {
    #[serde(rename="bids", alias="b")]
    bids: Option<serde_json::Value>,
    #[serde(rename="asks", alias="a")]
    asks: Option<serde_json::Value>,
    #[serde(rename="c", alias="c")]
    last_price: Option<String>,
}

async fn get_snapshot(ticker: &str, local_book: Arc<RwLock<LocalOrderBook>>) {
    let url = format!("https://api.binance.com/api/v3/depth?symbol={}USDT&limit=50", ticker.to_uppercase());
    let client = reqwest::Client::new();
    let response = client.get(url)
        .send()
        .await;

    match response {
        Ok(resp) => {
            match resp.json::<SnapshotResponse>().await {
                Ok(data) => {
                    // Обработка asks 
                    parse_asks(data.clone(), local_book.clone()).await;
                    parse_bids(data.clone(), local_book.clone()).await;
                }
                Err(e) => {
                    eprintln!("[Binace] Failed to get json data: {e}")
                }
            }
        }
        Err(e) => {
            eprintln!("[Binance]-error: {e}")
        }
    }
}

pub async fn connect(ticker: &str, _channel_type: &str, local_book: Arc<RwLock<LocalOrderBook>>) {
    get_snapshot(ticker, local_book.clone()).await;

    let url = Url::parse("wss://stream.binance.com:443/ws").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Binance] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Binance-Websocket] is running");

    let orderbook = format!("{}usdt@depth", ticker.to_lowercase());
    let price = format!("{}usdt@ticker", ticker.to_lowercase());

    write.send(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::json!({
            "method": "SUBSCRIBE",
            "params": [
                orderbook,
                price
            ]
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap();
        let local_order_book = local_book.clone();

        match data {
            Message::Text(text) => {
                fetch_data(text, local_order_book).await
            },
            _ => {}
        }
    });

    read_future.await;
}

async fn fetch_data(str_data: String, local_book: Arc<RwLock<LocalOrderBook>>) {
    let json = serde_json::from_str::<SnapshotResponse>(&str_data).unwrap();

    let mut book = {
        let current = local_book.read().await;
        current.clone()
    };

    if let Some(price_str) = json.last_price {
        let price = price_str.parse::<f64>().unwrap();
        book.snapshot.last_price = price;
    } 

    if let Some(serde_json::Value::Array(asks_array)) = json.asks {
        for asks in asks_array {
            if let serde_json::Value::Array(ask_vec) = asks {
                let mut price = 0.0;
                if let serde_json::Value::String(price_str) = &ask_vec[0] {
                    price = price_str.parse::<f64>().unwrap();
                }

                let mut volume = 0.0;
                if let serde_json::Value::String(vol_str) = &ask_vec[1] {
                    volume = vol_str.parse::<f64>().unwrap();
                }

                // Удаляем запись
                if volume == 0.0  {
                    book.snapshot.a.retain(|x| x.0 != price);
                } else {
                    // Заменяем volume
                    if let Some(x) = book.snapshot.a.iter_mut().find(|x| x.0 == price) {
                        x.1 = volume
                    } else {
                        // Добавляем запись
                        book.snapshot.a.push((price, volume));
                    }
                }
            }
        }
    }

    if let Some(serde_json::Value::Array(bids_array)) = json.bids {
        for bids in bids_array {
            if let serde_json::Value::Array(bid_vec) = bids {
                let mut price = 0.0;
                if let serde_json::Value::String(price_str) = &bid_vec[0] {
                    price = price_str.parse::<f64>().unwrap();
                }

                let mut volume = 0.0;
                if let serde_json::Value::String(vol_str) = &bid_vec[1] {
                    volume = vol_str.parse::<f64>().unwrap();
                }
                // Удаляем запись
                if volume == 0.0  {
                    book.snapshot.b.retain(|x| x.0 != price);
                } else {
                    // Заменяем volume
                    if let Some(x) = book.snapshot.b.iter_mut().find(|x| x.0 == price) {
                        x.1 = volume
                    } else {
                        book.snapshot.b.push((price, volume)); 
                    }
                }
            }
        }
    }

    book.snapshot.a.sort_by(|x, y| y.0.total_cmp(&x.0));

    let start = book.snapshot.a
        .iter()
        .position(|x| x.0 > book.snapshot.last_price)
        .unwrap_or(book.snapshot.a.len());

    book.snapshot.a = book.snapshot.a[start+1..].to_vec();

    book.snapshot.b.sort_by(|x, y| y.0.total_cmp(&x.0));

    let b_start = book.snapshot.b
        .iter()
        .position(|x| x.0 > book.snapshot.last_price)
        .unwrap_or(book.snapshot.b.len());

    book.snapshot.b = book.snapshot.b[0..b_start].to_vec();

    let mut book_lock = local_book.write().await;
    *book_lock = book
}

async fn parse_asks(data: SnapshotResponse, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        let current = local_book.read().await;
        current.clone()
    };

    if let Some(serde_json::Value::Array(asks_array)) = data.asks {
        for asks in asks_array {
            if let serde_json::Value::Array(ask_vec) = asks {
                let mut price = 0.0;
                if let serde_json::Value::String(price_str) = &ask_vec[0] {
                    price = price_str.parse::<f64>().unwrap();
                }

                let mut volume = 0.0;
                if let serde_json::Value::String(vol_str) = &ask_vec[1] {
                    volume = vol_str.parse::<f64>().unwrap();
                }

                book.snapshot.a.push((price, volume));
            }
        }
    }

    let mut book_lock = local_book.write().await;
    *book_lock = book
}

async fn parse_bids(data: SnapshotResponse, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        let current = local_book.read().await;
        current.clone()
    };

    if let Some(serde_json::Value::Array(bids_array)) = data.bids {
        for bids in bids_array {
            if let serde_json::Value::Array(bids_vec) = bids {
                let mut price = 0.0;
                if let serde_json::Value::String(price_str) = &bids_vec[0] {
                    price = price_str.parse::<f64>().unwrap();
                }

                let mut volume = 0.0;
                if let serde_json::Value::String(vol_str) = &bids_vec[1] {
                    volume = vol_str.parse::<f64>().unwrap();
                }

                book.snapshot.b.push((price, volume));
            }
        }
    }

    book.snapshot.a.sort_by(|a, b| b.0.total_cmp(&a.0));
    book.snapshot.b.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut book_lock = local_book.write().await;
    *book_lock = book
}