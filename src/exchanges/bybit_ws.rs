use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

use crate::exchanges::orderbook::{LocalOrderBook, Snapshot};

#[derive(Deserialize, Debug, Serialize)]
struct TickerResponse {
    #[serde(rename="result")]
    result: TickerResult
}

#[derive(Deserialize, Debug, Serialize)]
struct TickerResult {
    #[serde(rename="list")]
    list: Vec<Ticker>
}

#[derive(Deserialize, Debug, Serialize)]
struct Ticker {
    #[serde(rename="symbol")]
    symbol: String
}

#[derive(Deserialize, Debug, Serialize)]
struct OrderBookEvent {
    #[serde(rename="type")]
    order_type: Option<String>,
    #[serde(rename="data")]
    data: Option<OrderBookEventData>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct OrderBookEventData {
    #[serde(rename="s")]
    symbol: Option<String>,
    #[serde(rename="a")]
    asks: Option<Vec<Vec<String>>>,
    #[serde(rename="b")]
    bids: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEvent {
    #[serde(rename="data")]
    data: Option<TickerEventData>
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEventData {
    #[serde(rename="symbol")]
    symbol: String,
    #[serde(rename="lastPrice")]
    last_price: String
}

async fn get_tickers(channel_type: &str) -> Result<Vec<Ticker>> {
    let url = format!("https://api.bybit.com/v5/market/tickers?category={channel_type}");
    let client = reqwest::Client::new();
    let response = client.get(url)
        .send()
        .await?;

    let mut usdt_tickers = Vec::new();

    if response.status().is_success() {
        let json = response.json::<TickerResponse>().await?;
        usdt_tickers = json.result.list
            .into_iter()
            .filter(|ticker| ticker.symbol.ends_with("USDT"))
            .collect();
    }

    Ok(usdt_tickers)
}

pub async fn connect(channel_type: &str, local_book: Arc<RwLock<LocalOrderBook>>) {
    let tickers = get_tickers(channel_type).await.unwrap();

    let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Bybit] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Bybit-Websocket] is running");

    let mut params = Vec::new();

    for ticker in tickers {
        let symbol = ticker.symbol;
        let orderbook_str = format!("orderbook.50.{}", symbol.to_uppercase());
        let price_str = format!("tickers.{}", symbol.to_uppercase());

        params.push(orderbook_str);
        params.push(price_str);
    }

    for chunk in params.chunks(10) {
        write.send(Message::Text(
            serde_json::json!({
                "op": "subscribe",
                "channel_type": channel_type,
                "args": chunk
            }).to_string().into()
        )).await.unwrap();
    }

    let (tx_event, mut rx_event) = mpsc::channel::<TickerEvent>(1000);

    let order_book_cl = local_book.clone();
    tokio::spawn(async move {
        while let Some(event) = rx_event.recv().await {
            handle_price(event, order_book_cl.clone()).await;
        }
    });

    let read_future = read.for_each(|msg| async {
        let msg_type = msg.unwrap();

        match msg_type {
            Message::Text(channel) => {
                if channel.contains("orderbook.") {
                    let json = serde_json::from_str::<OrderBookEvent>(&channel).unwrap();
                    match json.order_type.as_deref() {
                        Some("snapshot") => handle_snapshot(json, local_book.clone()).await,
                        Some("delta") => handle_delta(json, local_book.clone()).await,
                        _ => {}
                    }
                }
                
                if channel.contains("tickers.") {
                    let json = serde_json::from_str::<TickerEvent>(&channel).unwrap();
                    tx_event.send(json).await.unwrap();
                }
            }
            _ => { 
                eprintln!("[Bybit-Websocket]: another type")
            }
        }
        
    });
    read_future.await;
}

async fn handle_snapshot(json: OrderBookEvent, local_book: Arc<RwLock<LocalOrderBook>>) {
    let book = {
        local_book.read().await.clone()
    };

    match json.data {
        Some(data) => {
            let ticker = data.symbol.unwrap().to_lowercase();

            // println!("{:?}", ticker);

            let asks = parse_levels(data.asks).await;
            let bids = parse_levels(data.bids).await;

            // Добавляем Asks/Bids в локальный orderbook
            book.books.insert(ticker, Snapshot {
                a: asks,
                b: bids,
                last_price: 0.0,
            });
        }
        _ => {}
    }

    let mut lock = local_book.write().await;
    *lock = book;
}

async fn handle_delta(json: OrderBookEvent, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut lock = local_book.write().await;

    match json.data {
        Some(data) => {
            let ticker = data.symbol.unwrap();
            let asks = parse_levels(data.asks).await;
            let bids = parse_levels(data.bids).await;

            lock.apply_snapshot_updates(&ticker, asks, bids).await;
        }
        _ => {}
    }
    drop(lock);
}

async fn parse_levels(data: Option<Vec<Vec<String>>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    
    match data {
        Some(array) => {
            for vec in array {
                let price = vec[0].parse::<f64>().expect("bad price");
                let volume = vec[1].parse::<f64>().expect("bad volume");
                
                let price_with_tick = (price * 100.0).round() as i64;
                values.insert(price_with_tick, volume);
            }
        }
        _ => {}
    }

    values
}

async fn handle_price(json: TickerEvent, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut lock = local_book.write().await;
    
    if let Some(data) = json.data {
        let ticker = data.symbol.to_lowercase();
        let last_price = data.last_price.parse::<f64>().expect("[Bybit-Websocket] bad price");

        lock.set_last_price(&ticker, last_price);
    }
    drop(lock);
}