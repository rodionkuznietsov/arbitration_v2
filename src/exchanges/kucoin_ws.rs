use std::{collections::BTreeMap, sync::Arc, time::Duration};

use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result};

use crate::exchanges::{orderbook::{LocalOrderBook, parse_levels__}, websocket::{Ticker, Websocket}};

#[derive(Debug, Deserialize, Serialize)]
struct ApiKeyResponse {
    #[serde(rename="data")]
    data: Data
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    #[serde(rename="token")]
    token: String
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerResponse {
    #[serde(rename="data")]
    data: Vec<Ticker>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="topic")]
    topic: String,
    data: Option<OrderBookEventData>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OrderBookEventData {
    #[serde(rename="asks")]
    asks: Vec<Vec<String>>,
    #[serde(rename="bids")]
    bids: Vec<Vec<String>>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TickerEvent {
    #[serde(rename="topic")]
    topic: String,
    #[serde(rename="data")]
    data: Option<TickerEventData>
}

#[derive(Deserialize, Debug, Serialize)]
struct TickerEventData {
    #[serde(rename="price")]
    last_price: String
}

#[derive(Debug)]
enum WebSocketStatus {
    Finished
}

enum WsCmd {
    Subscribe(String)
}

pub struct KuCoinWebsocket {
    title: String,
    enabled: bool,
    client: reqwest::Client,
    local_book: Arc<RwLock<LocalOrderBook>>,
    channel_type: String
}

impl KuCoinWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = "[KuCoin-Websocket]".to_string();
        let client = reqwest::Client::new();
        let local_book = LocalOrderBook::new();
        let channel_type = String::from("spot");

        let this = Arc::new(Self { title, enabled, client, local_book, channel_type});
        let this_cl = Arc::clone(&this);

        this_cl.connect("spot".to_string());
        this
    }

    async fn run_websocket(self: Arc<Self>, mut cmd_rx: mpsc::Receiver<WsCmd>) -> WebSocketStatus {
        let api_token = get_api_key().await.unwrap();

        let url = url::Url::parse(&format!("wss://ws-api-spot.kucoin.com?token={}", api_token)).unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect("[KuCoin] Failed to connect");
        let (mut write, mut read) = ws_stream.split();
        println!("🌐 {} is running", self.title);

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    // println!("Ticker: {}", ticker);

                    write.send(Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/spotMarket/level2Depth50:{}-USDT", ticker),
                            "response": true
                        }).to_string().into()
                    )).await.unwrap();

                    write.send(Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/market/ticker:{}-USDT", ticker),
                            "response": true
                        }).to_string().into()
                    )).await.unwrap();
                }
            }
        }

        let (queue_tx, queue_rx) = async_channel::bounded::<String>(10);
        let worker_count = 4;
        let this = Arc::clone(&self);
        tokio::spawn(async move {
            for _ in 0..=worker_count {
                let this = this.clone();
                let queue_rx = queue_rx.clone();
                
                while let Ok(channel) = queue_rx.recv().await {
                    let mut lock = self.local_book.write().await;
                    
                    if channel.contains("level2Depth50") {
                        let this = this.clone();
                        let json: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                        let result = this.handle_snapshot(json).await;
                        if let Some((ref ticker, asks, bids)) = result {
                            lock.apply_or_add_snapshot(ticker, asks, bids).await;
                        }
                    } else if channel.contains("ticker") {
                        let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                        let this = this.clone();
                        let result = this.handle_price(json).await;
                        if let Some((ref ticker, last_price)) = result {
                            // lock.set_last_price(ticker, last_price).await;
                        }
                    }

                    drop(lock);
                }
            }
        });

        while let Some(msg) = read.next().await {
            let msg_type = msg.unwrap();            
            match msg_type {
                Message::Text(channel) => {
                    match queue_tx.send(channel.clone()).await {
                        Ok(_) => {},
                        Err(e) => {
                            println!("{e}")
                        }
                    }
                }
                _ => {}
            }
        }

        WebSocketStatus::Finished
    }

    async fn ticker_formatted(self: Arc<Self>, topic: String) -> Option<String> {
        let regex = Regex::new(r"([a-zA-Z]+)-(USDT)$").unwrap();
        let ticker = regex
            .captures(&topic)
            .and_then(|c| c.get(0))
            .map(|m| m.as_str().to_string().replace("-", ""))?;

        Some(ticker.to_lowercase())
    }
}

impl Websocket for KuCoinWebsocket {
    type Snapshot = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>, _channel_type: String) {
        if !self.enabled {
            println!("{} enabled: false", self.title);
            return;
        } 

        let this = Arc::clone(&self);

        tokio::spawn(async move {
            let tickers = this.get_tickers(&this.channel_type).await.unwrap();

            for ticker_chunk in tickers.chunks(50) {
                let (cmd_tx, cmd_rx) = mpsc::channel(50);
                let this_cl = this.clone();

                for t in ticker_chunk {
                    let symbol = t.symbol.clone().unwrap().replace("-USDT", "");
                    cmd_tx.send(WsCmd::Subscribe(symbol)).await.expect(&format!("{} Failed to send symbol", this.title));
                }

                tokio::spawn(async move {
                    let result = this_cl.clone().run_websocket(cmd_rx).await;
                    println!("{}: {:?}", this_cl.title, result)
                });
            }
        });
    }

    fn get_local_book(self: Arc<Self>) -> Arc<RwLock<LocalOrderBook>> {
        self.local_book.clone()
    }

    async fn get_tickers(&self, _channel_type: &str) -> Result<Vec<super::websocket::Ticker>> {
        let url = "https://api.kucoin.com/api/v2/symbols";
        let response = self.client.get(url)
            .send()
            .await?;

        let mut retries = 0;

        loop {
            if response.status().is_success() {
                let json: TickerResponse = response.json().await?;

                let usdt_tickers: Vec<Ticker> = json.data.into_iter()
                    .filter(|t| t.symbol.clone().unwrap().ends_with("USDT"))
                    .collect();
                
                return Ok(usdt_tickers);
            } else {
                retries += 1;
                let delay = 2u64.pow(retries.min(5));
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }
        }
    }

    async fn handle_delta(self: Arc<Self>, _json: OrderBookEvent) {
        
    }

    async fn handle_price(self: Arc<Self>, json: TickerEvent) -> Option<(String, f64)> {
        // let this = Arc::clone(&self);
        let Some(data) = json.data else { return None };

        let ticker = self.ticker_formatted(json.topic).await;
        let last_price = match data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => return None 
        };

        let Some(ticker) = ticker else { return None };

        return Some((ticker, last_price));
    }

    async fn handle_snapshot(self: Arc<Self>, json: OrderBookEvent) -> Option<(String, BTreeMap<i64, f64>, BTreeMap<i64, f64>)>{
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(data) = json.data else { return None };

        let asks: BTreeMap<i64, f64> = parse_levels__(data.asks).await;
        let bids: BTreeMap<i64, f64> = parse_levels__(data.bids).await;
        let Some(ticker) = ticker else { return None };
        Some((ticker, asks, bids))
        
        // let lock = self.local_book.write().await;

        // let this = Arc::clone(&self);
        // match json.data {
        //     Some(data) => {
        //         let ticker = this.ticker_formatted(json.topic).await;                
        //         let asks = lock.parse_levels(data.asks).await;
        //         let bids = lock.parse_levels(data.bids).await;
                
        //         if let Some(ticker) = ticker {
        //             lock.apply_or_add_snapshot(&ticker, asks, bids).await;
        //         }
        //     }
        //     _ => {}
        // }
    }

}
 
/// Получает временный ключ доступа к WebSocket.
async fn get_api_key() -> Result<String> {
    let url = "https://api.kucoin.com/api/v1/bullet-public";
    
    let client = reqwest::Client::new();
    let response = client.post(url)  
        .send()
        .await?;

    let data = response.json::<ApiKeyResponse>().await?;
    let api_key = data.data.token;

    println!("[KuCoin-Rest] Api-Key успешно получен.");

    Ok(api_key)
}