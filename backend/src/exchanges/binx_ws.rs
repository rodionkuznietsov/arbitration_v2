use std::{collections::HashMap, io::Read, sync::Arc, time::Duration};
use flate2::read::MultiGzDecoder;
use futures_util::{SinkExt, StreamExt};

use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;

use crate::{exchanges::websocket::{Ticker, WebSocketStatus, Websocket, WsCmd}, models::orderbook::{BookEvent, OrderBookManager, Snapshot, SnapshotUi, parse_levels__}};

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
    title: String,
    enabled: bool,
    channel_type: String,
    client: reqwest::Client,
    sender_data: mpsc::Sender<BookEvent>,
    ticker_rx: async_channel::Receiver<(String, String)>,
    pub ticker_tx: async_channel::Sender<(String, String)>
}

impl BinXWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = String::from("[BinXWebsocket]");
        let channel_type = String::from("spot");
        let (sender_data, rx_data) = mpsc::channel(10);
        let client = reqwest::Client::new();
        let (ticker_tx, ticker_rx) = async_channel::bounded::<(String, String)>(1);

        let book_manager = OrderBookManager::new(rx_data);

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self {
            title, channel_type, sender_data,
            enabled, client, ticker_rx, ticker_tx
        });

        let this_cl = Arc::clone(&this);
        this_cl.connect();
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
            if !self.enabled {
                println!("{} enabled: false", this.title);
                return;
            } 

            let tickers = this.get_tickers(&this.channel_type).await;

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
            println!("{}: Reconnecting...", self.title);

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
                            println!("{}: {e}", self.title)
                        }
                    }
                }
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus {        
        let url = url::Url::parse("wss://open-api-ws.bingx.com/market").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.title));
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
                    println!("{}: {e}", self.title);
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
                                match self.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", self.title, e))
                                    }
                                }    
                            }
                        }

                        if channel.contains("@lastPrice") {
                            let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let result = self.clone().handle_price(json).await;

                            if let Some(event) = result {
                                match self.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("{}", format!("{}: {}", self.title, e))
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

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: tokio::sync::mpsc::Sender<SnapshotUi>) {
        if !self.enabled {
            return;
        }

        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(100);
            let this = self.clone();

            let ticker = if ticker.eq_ignore_ascii_case("ton") {
                "toncoin".to_string()
            } else {
                ticker
            };

            loop {
                let ticker = ticker.clone();
                
                match this.sender_data.send(BookEvent::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("{}: {}", this.title, e)
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

    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<super::websocket::Ticker>> {
        let url = format!("https://open-api.bingx.com/openApi/{channel_type}/v1/ticker/bookTicker");
        let response = self.client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = json.data
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();
        
        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<BookEvent> {
        let ticker = json.topic.replace("-USDT@depth50", "USDT").to_lowercase();
        let asks = parse_levels__(json.data.asks).await;
        let bids = parse_levels__(json.data.bids).await;

        Some(BookEvent::Snapshot { 
            ticker, 
            snapshot: Snapshot { a: asks, b: bids, last_price: 0.0, last_update_id: None }
        })
    }

    async fn handle_delta(self: Arc<Self>, _json: Self::Snapshot) -> Option<BookEvent> {
        todo!()
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<BookEvent> {
        let ticker = json.data.symbol.replace("-USDT", "USDT").to_lowercase();
        let last_price = match json.data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        Some(BookEvent::Price { ticker, last_price })
    }
}