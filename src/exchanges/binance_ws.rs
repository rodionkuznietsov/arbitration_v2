use std::{sync::Arc};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::{Message}};
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
    #[serde(rename="s")]
    symbol: Option<String>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ticker {
    pub symbol: String,
}

pub async fn get_tickers() -> Result<Vec<Ticker>> {
    let url = "https://api.binance.com/api/v3/ticker/bookTicker";
    let client = reqwest::Client::new();
    let response = client.get(url)
        .send()
        .await?;

    let mut usdt_tickers = Vec::new();

    if response.status().is_success() {
        let tickers: Vec<Ticker> = response.json().await?;
        usdt_tickers = tickers
            .into_iter()
            .filter(|ticker| ticker.symbol.ends_with("USDT"))
            .collect();
    }

    Ok(usdt_tickers)
}

pub async fn connect(local_book: Arc<RwLock<LocalOrderBook>>) {
    let tickers = get_tickers().await.unwrap();

    let url = Url::parse("wss://stream.binance.com:443/ws").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Binance] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Binance-Websocket] is running");

    let mut params = Vec::new();

    for ticker in tickers {
        let symbol = ticker.symbol;
        let orderbook_str = format!("{}@depth10", symbol.to_lowercase());
        let price_str = format!("{}@ticker", symbol.to_lowercase());

        params.push(orderbook_str);
        params.push(price_str);
    }
        
    write.send(tokio_tungstenite::tungstenite::Message::Text(
        serde_json::json!({
            "method": "SUBSCRIBE",
            "params": params[0..10]
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap();

        match data {
            Message::Text(text) => {
                let json = serde_json::from_str::<SnapshotResponse>(&text).unwrap();
                fetch_data(json, local_book.clone()).await
            }
            Message::Close(c) => {
                println!("[Binance-Websocket] is closing: {c:#?}")
            }
            _ => {}
        }
    });

    read_future.await;
}

async fn fetch_data(json: SnapshotResponse, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        local_book.read().await.clone()
    };

    if let Some(price_str) = json.last_price {
        book.snapshot.last_price = price_str.parse::<f64>().unwrap_or(0.0);
    }    

    if let Some(t) = json.symbol {
        book.snapshot.ticker = t.to_lowercase();
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
                    book.snapshot.a.retain(|x| x.1 != price);
                } else {
                    // Заменяем volume
                    if let Some(x) = book.snapshot.a.iter_mut().find(|x| x.1 == price) {
                        x.1 = volume
                    } else {
                        // Добавляем запись
                        book.snapshot.a.push((book.snapshot.ticker.clone(), price, volume));
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
                        // Добавляем запись
                        book.snapshot.b.push((price, volume));
                    }
                }
            }
        }
    }

    let mut lock = local_book.write().await;
    *lock = book;
}