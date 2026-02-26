use std::{io::Read, sync::Arc, time::Duration};
use async_trait::async_trait;
use flate2::read::MultiGzDecoder;
use futures_util::{SinkExt, StreamExt};

use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, broadcast, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::{exchanges::exchange_setup::ExchangeSetup, models::{self, exchange::ExchangeType, orderbook::{BookEvent, Snapshot, SnapshotUi}, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::{exchange_store::ExchangeStoreCMD, market_manager::ExchangeWebsocket}};
use crate::services::{websocket::Websocket, exchange_store::{parse_levels__}};

#[derive(Debug, Deserialize, Serialize)]
struct TickerResponse {
    data: Vec<Ticker>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookEvent {
    data: OrderBookEventData,
    #[serde(rename="dataType")]
    topic: String
}

#[derive(Debug, Deserialize, Serialize)]
struct OrderBookEventData {
    #[serde(rename="asks")]
    asks: Vec<Vec<String>>,
    #[serde(rename="bids")]
    bids: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    data: TickerEventData
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEventData {
    #[serde(rename="s")]
    symbol: String,
    #[serde(rename="c")]
    last_price: String
}

pub struct BinXWebsocket {
    setup: Arc<ExchangeSetup>
}

impl BinXWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let setup = ExchangeSetup::new(ExchangeType::BinX, enabled);
        let this = Arc::new(
            Self { 
                setup 
            }
        );

        this.clone().connect();
        this.clone().spawn_quote_updater();

        this
    }
}

impl Websocket for BinXWebsocket {
    type Snapshot = OrderBookEvent;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        let this = Arc::clone(&self);
        tokio::spawn(async move {
            if !self.setup.enabled {
                println!("{} enabled: false", this.setup.title);
                return;
            } 

            let tickers = this.get_tickers(&this.setup.channel_type).await;

            if let Some(tickers) = tickers {
                this.reconnect(&tickers).await;
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>) {        
        let notify = Arc::new(Notify::new());

        notify.notify_one();
        loop {
            notify.notified().await;
            println!("{}: Reconnecting...", self.setup.title);

            let token = CancellationToken::new();

            for chunk in tickers.chunks(80) {        
                let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCmd>(5);

                let this = self.clone();
                let notify = notify.clone();
                let token = token.clone();

                tokio::spawn(async move {
                    tokio::select! {
                        _ = token.cancelled() => {
                            return;
                        }

                        _ = this.clone().run_websocket(&mut cmd_rx) => {
                            token.cancel();
                        }
                    };

                    tokio::time::sleep(Duration::from_secs(5)).await;
                    notify.notify_one();
                });

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    match cmd_tx.send(WsCmd::Subscribe(symbol)).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{}: {e}", self.setup.title)
                        }
                    }
                }
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus {        
        let url = url::Url::parse("wss://open-api-ws.bingx.com/market").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.setup.title));
        let (mut write, mut read) = ws_stream.split();

        println!("🌐 [BinX-Websocket] is running");
        
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    let orderbook = format!("{}@depth50", ticker.to_uppercase());
                    write.send(Message::Text(
                        serde_json::json!({
                            "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
                            "reqType": "sub",
                            "dataType": orderbook
                        }).to_string().into()
                    )).await.unwrap();
    
                    let price = format!("{}@lastPrice", ticker.to_uppercase());
                    write.send(Message::Text(
                        serde_json::json!({
                            "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
                            "reqType": "sub",
                            "dataType": price
                        }).to_string().into()
                    )).await.unwrap();
                }
            }
        }

        while let Some(msg) = read.next().await {
            let msg_type = match msg {
                Ok(m) => Some(m),
                Err(e) => {
                    println!("{}: {e}", self.setup.title);
                    None
                }
            };

            if let Some(msg_type) = msg_type {
                match msg_type {
                    Message::Binary(binary) => {
                        let mut d = MultiGzDecoder::new(&*binary);
                        let mut channel = String::new();
                        d.read_to_string(&mut channel).unwrap();

                        if channel.contains("@depth50") {
                            let json: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                            let result = self.clone().handle_snapshot(json).await;
                            if let Some(event) = result {
                                match self.setup.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", self.setup.title, e))
                                    }
                                }    
                            }
                        }

                        if channel.contains("@lastPrice") {
                            let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let result = self.clone().handle_price(json).await;

                            if let Some(event) = result {
                                match self.setup.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", self.setup.title, e))
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

    async fn get_last_snapshot(self: Arc<Self>, snapshot_tx: tokio::sync::mpsc::Sender<SnapshotUi>) {
        if !self.setup.enabled {
            return;
        }

        while let Ok((_uuid, ticker)) = self.setup.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(100);
            let this = self.clone();

            let ticker = if ticker.eq_ignore_ascii_case("ton") {
                "toncoin".to_string()
            } else {
                ticker
            };

            loop {
                let ticker = ticker.clone();
                
                match this.setup.sender_data.send(ExchangeStoreCMD::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("{}: {}", this.setup.title, e)
                    },
                };

                tokio::select! {
                    data = rx.recv() => {
                        if let Some(snapshot_ui) = data {
                            if let Some(snapshot) = snapshot_ui {
                                match snapshot_tx.send(snapshot).await {
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

    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<models::websocket::Ticker>> {
        let url = format!("https://open-api.bingx.com/openApi/{channel_type}/v1/ticker/bookTicker");
        let response = self.setup.client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = json.data
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();
        
        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<ExchangeStoreCMD> {
        let ticker = json.topic.replace("-USDT@depth50", "USDT").to_lowercase();
        let asks = parse_levels__(json.data.asks).await;
        let bids = parse_levels__(json.data.bids).await;

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
        let ticker = json.data.symbol.replace("-USDT", "USDT").to_lowercase();
        let last_price = match json.data.last_price.parse::<f64>() {
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

#[async_trait]
impl ExchangeWebsocket for BinXWebsocket {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)> {
        self.setup.ticker_tx.clone()
    }

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>) {
        self.get_last_snapshot(snapshot_tx).await
    }

    fn spawn_quote_updater(
        self: Arc<Self>
    ) {
        let mut rx = self.setup.books_updates.subscribe();
        let title = self.setup.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        // if self.setup.sender_data.send(ExchangeStoreCMD::Quote { ticker }).await.is_err() {
                        //     continue;
                        // }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал спреда закрыт", title);
                        break;
                    }
                }
            }
        });
    }
}