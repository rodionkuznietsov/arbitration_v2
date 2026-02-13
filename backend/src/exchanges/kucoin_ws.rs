use std::{collections::HashMap, sync::{Arc}, time::Duration};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result};
use tokio_util::sync::CancellationToken;

use crate::{exchanges::{websocket::{Ticker, WebSocketStatus, Websocket, WsCmd}}, models::orderbook::SnapshotUi};
use crate::models::orderbook::{BookEvent, OrderBookManager, Snapshot, parse_levels__};

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

pub struct KuCoinWebsocket {
    title: String,
    enabled: bool,
    client: reqwest::Client,
    channel_type: String,
    sender_data: mpsc::Sender<BookEvent>,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    ticker_rx: async_channel::Receiver<(String, String)>,
}

impl KuCoinWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = "[KuCoin-Websocket]".to_string();
        let client = reqwest::Client::new();
        let channel_type = String::from("spot");
        let (sender_data, rx_data) = mpsc::channel(50);
        let (ticker_tx, ticker_rx) = async_channel::bounded::<(String, String)>(1);

        let book_manager = OrderBookManager::new(rx_data);

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self { 
            title, enabled, client, channel_type, 
            sender_data, ticker_tx, ticker_rx
        });
        let this_cl = Arc::clone(&this);
        this_cl.connect();

        this
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
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        tokio::spawn(async move {
            if !self.enabled {
                println!("{} enabled: false", self.title);
                return ;
            }

            let this = self.clone();
            let tickers = self.get_tickers(&self.channel_type).await;
            
            // Обрабатываем подписку на все токены
            if let Some(tickers) = tickers {
                this.reconnect(&tickers).await;
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>) {                        
        let notify = Arc::new(Notify::new());
        
        notify.notify_one(); // Инициализируем запуск
        loop {    
            notify.notified().await;
            println!("{}: Reconnecting...", self.title);
            
            let token = CancellationToken::new();

            for chunk in tickers.chunks(50) {
                let (cmd_tx, cmd_rx) = mpsc::channel::<WsCmd>(50);

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap().replace("-USDT", "");
                    match cmd_tx.send(WsCmd::Subscribe(symbol)).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{}: {{cmd_tx_event_recconect}} {e}", self.title)
                        }
                    }
                }
            
                let this = self.clone();
                let mut cmd_rx = cmd_rx;
                let token = token.clone();
                let notify = notify.clone();

                // На каждый чанк запускаем новое подключение к вебсокету
                tokio::spawn(async move {
                    tokio::select! {
                        _ = token.cancelled() => {
                            return;
                        },
                        _ = this.clone().run_websocket(&mut cmd_rx) => {
                            token.cancel(); // Удаляем текущую задачу
                        }
                    };

                    tokio::time::sleep(Duration::from_secs(1)).await; 
                    notify.notify_one(); // Реконектемся
                });
            }
        }
    }
    
    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus {
        let api_token = get_api_key().await.unwrap();

        let url = url::Url::parse(&format!("wss://ws-api-spot.kucoin.com?token={}", api_token)).unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.title));
        let (mut write, mut read) = ws_stream.split();
        println!("🌐 {} is running", self.title);

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
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
                Err(e) => {
                    println!("{}: {{msg_event_run_websocket_read}} {e}", this.title);
                    None
                }
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
                                        println!("{}: {{sender_data_event_level2Depth50}} {e}", this.title)
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
                                        println!("{}: {{sender_data_event_ticker}} {e}", this.title)
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

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::UnboundedSender<SnapshotUi>) {
        if !self.enabled {
            return ;
        }

        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(100);    
            let this = Arc::clone(&self);

            loop {
                let ticker = ticker.clone();
                
                match this.sender_data.send(BookEvent::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("{}: {{sender_data_event_get_book}} {e}", this.title)
                    },
                };

                tokio::select! {
                    data = rx.recv() => {
                        if let Some(snapshot_ui) = data {
                            if let Some(snapshot) = snapshot_ui {
                                match snapshot_tx.send(snapshot) {
                                    Ok(_) => {}
                                    Err(_) => {}
                                }
                            }
                        }
                    }

                    _ = tokio::time::sleep(Duration::from_millis(750)) => {}
                }
            }
        }
    }
    
    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<Ticker>> {
        let url = "https://api.kucoin.com/api/v1/market/allTickers";
        let response = self.client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = json.data.ticker
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();

        Some(usdt_tickers)
    }
    
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<BookEvent> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let asks = parse_levels__(data.asks).await;
        let bids = parse_levels__(data.bids).await;

        Some(BookEvent::Snapshot { 
            ticker, 
            snapshot: Snapshot { a: asks, b: bids, last_price: 0.0, last_update_id: None } 
        })
    }
    
    async fn handle_delta(self: Arc<Self>, _json: Self::Snapshot) -> Option<BookEvent> {
        todo!()
    }
    
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<BookEvent> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let last_price = match data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        Some(BookEvent::Price { ticker, last_price })
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