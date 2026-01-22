use std::{collections::BTreeMap, sync::Arc, thread::sleep, time::Duration};
use anyhow::Result;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, stream::{self, SplitStream}};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncRead, AsyncWrite}, net::TcpStream, sync::RwLock};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use url::Url;

use crate::exchanges::orderbook::{LocalOrderBook, Snapshot};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ticker {
    pub symbol: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="a")]
    asks: Vec<Vec<String>>,
    #[serde(rename="b")]
    bids: Vec<Vec<String>>,
    #[serde(rename="s")]
    symbol: String
}

#[derive(Debug, Deserialize, Serialize)]
struct SnapshotResponse {
    #[serde(rename="asks")]
    asks: Vec<Vec<String>>,
    #[serde(rename="bids")]
    bids: Vec<Vec<String>>,
    #[serde(rename="s")]
    symbol: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerEvent {
    #[serde(rename="s")]
    symbol: String,
    #[serde(rename="c")]
    last_price: String,
}

pub async fn get_tickers(client: &reqwest::Client) -> Result<Vec<Ticker>> {
    let url = "https://api.binance.us/api/v3/ticker/bookTicker";
    let mut usdt_tickers = Vec::new();

    let mut retries = 0;

    loop {
        let response = client
            .get(url)
            .header("User-Agent", "arbitrage-bot/0.1")
            .send()
            .await?;

        if response.status() == 418 {
            retries += 1;
            let delay = 2u64.pow(retries.min(5));
            // println!("[BinanceWs] 418 received, retrying in {}s", delay);
            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        } else if response.status() == 200 {
            let tickers: Vec<Ticker> = response.json().await?;
            usdt_tickers = tickers
                .into_iter()
                .filter(|ticker| ticker.symbol.ends_with("USDT"))
                .collect();
            return Ok(usdt_tickers);
        }
    }
}

async fn get_spot_snapshot(symbol: String, client: &reqwest::Client) -> Result<Option<SnapshotResponse>> {
    let url = format!("https://api.binance.com/api/v3/depth?symbol={}&limit=1000", symbol);
    let mut snapshot = None;
    let mut retries = 0;

    loop {
        let response = client.get(&url)
            .send()
            .await?;

        if response.status() == 418 {
            retries += 1;
            let delay = 2u64.pow(retries.min(5));
            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        } else if response.status() == 200 {
            let mut json: SnapshotResponse = response.json().await?;
            json.symbol = Some(symbol.clone());
            snapshot = Some(json);
            
            break;
        }
    }
  
    Ok(snapshot)
}

pub async fn connect(local_book: Arc<RwLock<LocalOrderBook>>) {
    let client = reqwest::Client::new();
    let tickers = get_tickers(&client).await.unwrap();

    let url = Url::parse("wss://stream.binance.com:443/ws").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Binance] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Binance-Websocket] is running");

    let mut params = Vec::new();

    for ticker in &tickers {
        let symbol = ticker.symbol.to_lowercase().clone();
        let orderbook_str = format!("{}@depth@100ms", symbol);
        let last_price_str = format!("{}@ticker", symbol);

        params.push(orderbook_str);
        params.push(last_price_str);
    }

    for chunk in params.chunks(200) {
        write.send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::json!({
                "method": "SUBSCRIBE",
                "params": chunk
            }).to_string().into()
        )).await.unwrap();
    }

    let (sender, receiver) = async_channel::unbounded::<SnapshotResponse>();

    tokio::spawn(async move {
        ws_read_loop(read, local_book, receiver).await;
    });

    let client = Arc::new(client);
    let mut snapshots = stream::iter(tickers)
        .map(move |ticker| {
            let symbol = ticker.symbol.clone();
            let client = Arc::clone(&client);
            async move {
                get_spot_snapshot(symbol, &client).await
            }
        })
        .buffer_unordered(3); // 3 одновременных запросов

    tokio::spawn({
        async move {
            while let Some(s) = snapshots.next().await {
                let snapshot = s.unwrap();
                if let Some(snap) = snapshot {
                    sender.send(snap).await.expect("[BinanceWS] Failed to send snapshot")
                }
            }
        }
    });
}

async fn ws_read_loop<S>(
    mut read: SplitStream<WebSocketStream<S>>, 
    local_book: Arc<RwLock<LocalOrderBook>>,
    receiver: async_channel::Receiver<SnapshotResponse>
) 
where 
    S: AsyncRead + AsyncWrite + Unpin,
{
    while let Some(msg) = read.next().await {
        let msg_type = match msg {
            Ok(m) => m,
            Err(e) => {
                println!("[Binance-Websocket] read error: {e}");
                return;
            }
        };

        match msg_type {
            Message::Text(channel) => {
                
                if channel.contains("depthUpdate") {   
                    let json: OrderBookEvent = serde_json::from_str(&channel).unwrap();

                    tokio::select! {
                        Ok(snapshot) = receiver.recv() => {
                            handle_snapshot(snapshot, local_book.clone()).await;
                        }
                        _ = handle_delta(json, local_book.clone()) => {}
                    }
                }
                
                if channel.contains("24hrTicker") {
                    let json = serde_json::from_str::<TickerEvent>(&channel).unwrap();
                    handle_price(json, local_book.clone()).await;
                }
            }, 
            _ => {}
        }
    }
}


async fn handle_snapshot(json: SnapshotResponse, local_book: Arc<RwLock<LocalOrderBook>>) {
    let book = {
        local_book.read().await.clone()
    };

    if let Some(t) = json.symbol {
        let asks = book.parse_levels(json.asks).await;
        let bids = book.parse_levels(json.bids).await;

        book.books.insert(t.to_lowercase(), Snapshot {
            a: asks,
            b: bids,
            last_price: 0.0
        });
    }

    let mut lock = local_book.write().await;
    *lock = book;
    drop(lock);
}

async fn handle_delta(json: OrderBookEvent, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut lock = local_book.write().await;

    let ticker = json.symbol.clone().to_lowercase();
    let asks = lock.parse_levels(json.asks).await;
    let bids = lock.parse_levels(json.bids).await;

    lock.apply_snapshot_updates(&ticker, asks, bids).await;
    drop(lock);
}

async fn handle_price(json: TickerEvent, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut lock = local_book.write().await;

    let ticker = json.symbol;
    let last_price = json.last_price.parse::<f64>().expect("[Bybit-Websocket] Bad price");

    lock.set_last_price(&ticker.to_lowercase(), last_price);
    drop(lock);
}