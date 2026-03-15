use std::{collections::VecDeque, sync::Arc, time::{Duration, Instant}};
use dashmap::DashSet;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::{models::{exchange::ExchangeType, websocket::{Symbol, Ticker, WebSocketStatus, WsCmd}}, services::{data_aggregator::AggregatorCommand, exchange_aggregator::ExchangeStoreCMD, exchange_setup::{ExchangeSetup, ExchangeWebsocket}}};
use crate::models::orderbook::{BookEvent, Snapshot};
use crate::services::{websocket::Websocket, exchange_aggregator::{parse_levels__}};

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
    last_price: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Volume {
    #[serde(rename="data")]
    data: VolumeData
}

#[derive(Debug, Deserialize, Serialize)]
struct VolumeData {
    data: VolumeValueData
}

#[derive(Debug, Deserialize, Serialize)]
struct VolumeValueData {
    #[serde(rename="marketChange24h")]
    market_change24h: Volume24hr,
    #[serde(rename="baseCurrency")]
    symbol: Symbol,
}

#[derive(Debug, Deserialize, Serialize)]
struct Volume24hr {
    #[serde(rename="volValue")]
    vol: Option<f64>
}

pub struct KuCoinWebsocket {
    setup: Arc<ExchangeSetup>,
    pending_ws_tokens: Mutex<VecDeque<String>>,
    client: reqwest::Client
}

impl KuCoinWebsocket {
    pub fn new(
        enabled: bool,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    ) -> Arc<Self> {
        let setup = ExchangeSetup::new(
            ExchangeType::KuCoin, 
            enabled,
            aggregator_tx.clone()
        );

        let pending_ws_tokens = Mutex::new(VecDeque::new());
        let client = reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build().unwrap();

        let this = Arc::new(
            Self { 
                setup,
                pending_ws_tokens,
                client
            }
        );

        this.clone().connect();
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

    async fn handle_volume24hr(self: Arc<Self>, json: Volume) -> Option<ExchangeStoreCMD> {
        let data = json.data;
        let result = data.data;
        let ticker = format!("{}usdt", result.symbol.to_lowercase());
        let Some(volume) = result.market_change24h.vol else { return None };
        Some(ExchangeStoreCMD::Event(BookEvent::Volume24hr { ticker: ticker.to_lowercase(), volume }))
    }
}
 
impl Websocket for KuCoinWebsocket {
    type Snapshot = OrderBookEvent;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        if !self.setup.enabled {
            warn!("{} is disabled", self.setup.title);
            return;
        }
        
        tokio::spawn({
            let this = self.clone();
            
            async move {
                for _ in 0..35 {
                    let api_key = get_api_key(&this.client).await.unwrap();
                    let mut lock = this.pending_ws_tokens.lock().await;
                    lock.push_back(api_key);
                }

                let this = self.clone();
                let tickers = self.get_tickers(&self.setup.channel_type).await;
                
                // Обрабатываем подписку на все токены
                if let Some(tickers) = tickers {
                    this.reconnect(&tickers).await;
                }
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>) {                        
        let notify = Arc::new(Notify::new());
        
        notify.notify_one(); // Инициализируем запуск
        loop {    
            notify.notified().await;
            println!("{}: Reconnecting...", self.setup.title);
            
            let token = CancellationToken::new();

            for chunk in tickers.chunks(30) {
                let (cmd_tx, cmd_rx) = mpsc::channel::<WsCmd>(30);

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap().to_uppercase();
                    let message1 = Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/spotMarket/level2Depth50:{}", symbol),
                            "response": true
                        }).to_string().into()
                    );

                    let message2 = Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/market/ticker:{}", symbol),
                            "response": true,
                            "privateChannel": false,
                        }).to_string().into()
                    );

                    let message3 = Message::Text(
                        serde_json::json!({
                            "type": "subscribe",
                            "topic": format!("/market/snapshot:{}", symbol),
                            "response": true,
                            "privateChannel": false,
                        }).to_string().into()
                    );

                    let messages = vec![message1, message2, message3];

                    match cmd_tx.send(WsCmd::Subscribe(messages)).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{}: {{cmd_tx_event_recconect}} {e}", self.setup.title)
                        }
                    }
                }

                // Извлекаем ключ
                let api_token = {
                    let mut api_key = self.pending_ws_tokens.lock().await;
                    api_key.pop_front()
                };

                let this = self.clone();
                let mut cmd_rx = cmd_rx;
                let token = token.clone();
                let notify = notify.clone();

                if let Some(key) = api_token {
                    // На каждый чанк запускаем новое подключение к вебсокету
                    // tokio::spawn(async move {
                    //     tokio::select! {
                    //         _ = token.cancelled() => {
                    //             return;
                    //         },
                    //         _ = this.setup.clone().run_ws(
                    //             format!("wss://ws-api-spot.kucoin.com?token={}", key),
                    //             &mut cmd_rx,
                    //         ) => {
                    //             token.cancel(); // Удаляем текущую задачу
                    //         }
                    //     };

                    //     tokio::time::sleep(Duration::from_secs(1)).await; 
                    //     notify.notify_one(); // Реконектемся
                    // });
                }
            }
        }
    }
    
    async fn run_websocket(
        self: Arc<Self>, 
        cmd_rx: &mut mpsc::Receiver<WsCmd>,
        api_token: Option<String>
    ) -> WebSocketStatus {
        if let Some(key) = api_token {
            let url = url::Url::parse(&format!("wss://ws-api-spot.kucoin.com?token={}", key)).unwrap();
            let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.setup.title));
            let (mut write, mut read) = ws_stream.split();
            println!("🌐 {} is running", self.setup.title);

            // while let Some(cmd) = cmd_rx.recv().await {
            //     match cmd {
            //         WsCmd::Subscribe(ticker) => {
            //             write.send(Message::Text(
            //                 serde_json::json!({
            //                     "type": "subscribe",
            //                     "topic": format!("/spotMarket/level2Depth50:{}", ticker.to_uppercase()),
            //                     "response": true
            //                 }).to_string().into()
            //             )).await.unwrap();

            //             write.send(Message::Text(
            //                 serde_json::json!({
            //                     "type": "subscribe",
            //                     "topic": format!("/market/ticker:{}", ticker.to_uppercase()),
            //                     "response": true,
            //                     "privateChannel": false,
            //                 }).to_string().into()
            //             )).await.unwrap();

            //             write.send(Message::Text(
            //                 serde_json::json!({
            //                     "type": "subscribe",
            //                     "topic": format!("/market/snapshot:{}", ticker.to_uppercase()),
            //                     "response": true,
            //                     "privateChannel": false,
            //                 }).to_string().into()
            //             )).await.unwrap();
            //         }
            //     }
            // }

            let this = Arc::clone(&self);
            while let Some(msg) = read.next().await {
                let msg_type = match msg {
                    Ok(m) => Some(m),
                    Err(e) => {
                        println!("{}: {{msg_event_run_websocket_read}} {e}", this.setup.title);
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
                                    match this.setup.sender_data.send(event).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            println!("{}: {{sender_data_event_level2Depth50}} {e}", this.setup.title)
                                        }
                                    }
                                } 
                            } 
                            
                            if channel.contains("ticker") {
                                let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                                let result = this.clone().handle_price(json).await;
                                if let Some(event) = result {
                                    match this.setup.sender_data.send(event).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            println!("{}: {{sender_data_event_ticker}} {e}", this.setup.title)
                                        }
                                    }
                                }
                            }

                            if channel.contains("snapshot") {
                                let json: Option<Volume> = match serde_json::from_str(&channel) {
                                    Ok(e) => e,
                                    Err(_) => None
                                };

                                if let Some(result) = json {
                                    let result = this.clone().handle_volume24hr(result).await;
                                    if let Some(event) = result {
                                        match this.setup.sender_data.send(event).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("{} -> {}", this.setup.title, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        WebSocketStatus::Finished
    }
    
    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<Ticker>> {
        let url = "https://api.kucoin.com/api/v1/market/allTickers";
        let response = self.setup.client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = json.data.ticker
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();

        Some(usdt_tickers)
    }
    
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<ExchangeStoreCMD> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let asks = parse_levels__(data.asks).await;
        let bids = parse_levels__(data.bids).await;

        Some(ExchangeStoreCMD::Event(
            BookEvent::Snapshot { 
                ticker, 
                snapshot: Snapshot { 
                    a: asks, 
                    b: bids, 
                    last_price: 0.0, 
                    last_update_id: None 
                } 
            }
        ))
    }
    
    async fn handle_delta(self: Arc<Self>, _json: Self::Snapshot) -> Option<ExchangeStoreCMD> {
        todo!()
    }
    
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<ExchangeStoreCMD> {
        let Some(data) = json.data else { return None};
        let ticker = self.ticker_formatted(json.topic).await;
        let Some(ticker) = ticker else { return None };
        let last_price = match data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        Some(ExchangeStoreCMD::Event(
            BookEvent::Price { 
                ticker, 
                last_price 
            }
        ))
    }
}

/// Получает временный ключ доступа к WebSocket.
async fn get_api_key(client: &reqwest::Client) -> Result<String> {
    let url = "https://api.kucoin.com/api/v1/bullet-public";
    
    let start = Instant::now();
    let response = client.post(url)  
        .send()
        .await?;

    let data = response.json::<ApiKeyResponse>().await?;
    let api_key = data.data.token;

    println!("KuCoin Millis: {}", start.elapsed().as_millis());

    // println!("[KuCoin-Rest] Api-Key успешно получен.");

    Ok(api_key)
}