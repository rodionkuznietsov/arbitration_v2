use std::{collections::{HashMap}, sync::Arc, time::Duration};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result};

use crate::exchanges::{orderbook::{BookEvent, OrderBookManager, Snapshot, SnapshotUi, parse_levels__}, websocket::Ticker};

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
    data: TickerData
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerData {
    ticker: Vec<Ticker>
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

#[derive(Debug, PartialEq)]
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
    channel_type: String,
    sender_data: async_channel::Sender<BookEvent>,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    ticker_rx: async_channel::Receiver<(String, String)>,
}

impl KuCoinWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        
        let title = "[KuCoin-Websocket]".to_string();
        let client = reqwest::Client::new();
        let channel_type = String::from("spot");
        let (sender_data, rx_data) = async_channel::bounded(100);
        let (ticker_tx, mut ticker_rx) = async_channel::bounded::<(String, String)>(1);

        let book_manager = OrderBookManager {
            books: HashMap::new(),
            rx: rx_data
        };

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self { 
            title, enabled, client, channel_type, 
            sender_data, ticker_tx, ticker_rx
        });
        let this_cl = Arc::clone(&this);
        this_cl.handle_tickers();

        this
    }

    pub async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::UnboundedSender<SnapshotUi>) {        
        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(100);    
            let this = Arc::clone(&self);

            loop {
                let ticker = ticker.clone();
                this.sender_data.send(BookEvent::GetBook { ticker, reply: tx.clone() }).await.unwrap();

                tokio::select! {
                    data = rx.recv() => {
                        if let Some(snapshot_ui) = data {
                            if let Some(snapshot) = snapshot_ui {
                                // println!("{}: {}", uuid, snapshot.last_price);
                                snapshot_tx.send(snapshot).unwrap();
                            }
                        }
                    }

                    _ = tokio::time::sleep(Duration::from_millis(750)) => {}
                }
            }
        }
    }

    fn handle_tickers(self: Arc<Self>) {
        tokio::spawn(async move {
            if !self.enabled {
                println!("{} enabled: false", self.title);
                return ;
            }

            let this = self.clone();
            let tickers = self.get_tickers().await;
            
            // Обрабатываем подписку на все токены
            if let Some(tickers) = tickers {
                for chunk in tickers.chunks(50) {
                    let (cmd_tx, cmd_rx) = async_channel::bounded(50);

                    for ticker in chunk {
                        let symbol = ticker.symbol.clone().unwrap().replace("-USDT", "");
                        cmd_tx.send(WsCmd::Subscribe(symbol)).await.unwrap();
                    }
                    
                    // На каждый чанк запускаем новое подключение к вебсокету
                    let this = this.clone();
                    tokio::spawn(async move {
                        loop {
                            let status = this.clone().run_websocket(cmd_rx.clone()).await;

                            match status {
                                WebSocketStatus::Finished => {
                                    println!("{}: Reconnecting...", this.title);
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                    continue;
                                } 
                            }
                        }
                    });
                }
            }
        });
    }

    async fn run_websocket(self: Arc<Self>, mut cmd_rx: async_channel::Receiver<WsCmd>) -> WebSocketStatus {
        let api_token = get_api_key().await.unwrap();

        let url = url::Url::parse(&format!("wss://ws-api-spot.kucoin.com?token={}", api_token)).unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect("[KuCoin] Failed to connect");
        let (mut write, mut read) = ws_stream.split();
        println!("🌐 {} is running", self.title);

        while let Ok(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    // println!("Ticker: {}", ticker);

                    write.send(Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/spotMarket/level2Depth50:{}-USDT", ticker.to_uppercase()),
                            "response": true
                        }).to_string().into()
                    )).await.unwrap();

                    write.send(Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/market/ticker:{}-USDT", ticker.to_uppercase()),
                            "response": true
                        }).to_string().into()
                    )).await.unwrap();
                }
            }
        }

        let this = Arc::clone(&self);
        while let Some(msg) = read.next().await {
            let msg_type = match msg {
                Ok(m) => Some(m),
                Err(_) => None
            };            

            if let Some(msg_type) = msg_type {
                match msg_type {
                    Message::Text(channel) => {
                        if channel.contains("level2Depth50") {
                            let json: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                            let result = this.clone().handle_snapshot(json).await;
                            if let Some(event) = result {
                                match this.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", this.title, e))
                                    }
                                }
                            } 
                        } 
                        
                        if channel.contains("ticker") {
                            let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let result = this.clone().handle_price(json).await;
                            if let Some(event) = result {
                                match this.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", this.title, e))
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
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

    async fn handle_snapshot(self: Arc<Self>, json: OrderBookEvent) -> Option<BookEvent> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let asks = parse_levels__(data.asks).await;
        let bids = parse_levels__(data.bids).await;

        Some(BookEvent::Snapshot { ticker, snapshot: Snapshot { a: asks, b: bids, last_price: 0.0 } })
    }

    async fn handle_price(self: Arc<Self>, json: TickerEvent) -> Option<BookEvent> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let last_price = match data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        Some(BookEvent::Price { ticker, last_price })
    }

    async fn get_tickers(self: Arc<Self>) -> Option<Vec<Ticker>>  {
        let url = "https://api.kucoin.com/api/v1/market/allTickers";
        let client = reqwest::Client::new();
        let response = client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = json.data.ticker
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();

        Some(usdt_tickers)
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