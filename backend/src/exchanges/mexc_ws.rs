use std::{collections::HashMap, sync::Arc, time::{Duration}};
use prost::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, Semaphore, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message as TungsteniteMessage};
use futures_util::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::warn;
use url::Url;

use crate::{
    exchanges::{orderbook::{BookEvent, Delta, OrderBookManager, Snapshot, parse_levels__}, 
    websocket::{Ticker, WebSocketStatus, Websocket, WsCmd}}, 
    mexc_orderbook::{Event, OrderBookEvent, TickerEvent}
};

#[derive(Debug, Deserialize, Serialize)]
pub struct SnapshotResponse {
    symbol: Option<String>,
    asks: Vec<Vec<String>>,
    bids: Vec<Vec<String>>,
    #[serde(rename="lastUpdateId")]
    last_update_id: u64
}

#[derive(Debug)]
pub struct TickerEventWithSymbol {
    data: TickerEvent,
    symbol: Option<String> 
}

pub struct MexcWebsocket {
    title: String,
    enabled: bool,
    channel_type: String,
    client: reqwest::Client,
    sender_data: mpsc::Sender<BookEvent>,
    ticker_rx: async_channel::Receiver<(String, String)>,
    pub ticker_tx: async_channel::Sender<(String, String)>,
}

impl MexcWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = String::from("[MexcWebsocket]");
        let (sender_data, rx_data) = mpsc::channel(10);
        let (ticker_tx, ticker_rx) = async_channel::bounded::<(String, String)>(1);
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();

        let book_manager = OrderBookManager::new(rx_data);

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, sender_data, client,
            ticker_tx, ticker_rx, channel_type
        });

        let this_cl = this.clone();
        this_cl.connect();

        this
    }   

    async fn get_ticker_snapshot_with_retry(self: Arc<Self>, ticker: &str) -> Option<SnapshotResponse> {
        let this = self.clone();
        let notify = Arc::new(Notify::new());
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);
        let mut _snapshot = None;
                
        println!("Ticker: {}", ticker);

        notify.notify_one();
        loop {
            notify.notified().await;
            
            let result = this.clone().get_ticker_snapshot(ticker).await;
            if let Some((status, snap)) = result {
                if status == 429 {
                    delay = std::cmp::min(delay * 2, max_delay);
                    println!("{}: Reconnecting to ticker: {} in {} secs", this.title, ticker, delay.as_secs_f64());
                    tokio::time::sleep(delay).await;
                    notify.notify_one();
                } else {
                    _snapshot = Some(snap);
                    break;
                }
            } 
        }

        _snapshot
    }

    async fn get_ticker_snapshot(self: Arc<Self>, ticker: &str) -> Option<(u16, SnapshotResponse)> {
        let url = format!("https://api.mexc.com//api/v3/depth?symbol={ticker}&limit=1000");
        let response = self.client.get(url).send().await;        
        let Ok(response) = response else { return None };
        let status = response.status().as_u16();
        let Ok(mut json) = response.json::<SnapshotResponse>().await else { return None };
        json.symbol = Some(ticker.to_string());
        Some((status, json))
    }
}

impl Websocket for MexcWebsocket {
    type Snapshot = SnapshotResponse;
    type Delta = OrderBookEvent;
    type Price = TickerEventWithSymbol;

    fn connect(self: std::sync::Arc<Self>) {
        tokio::spawn(async move {
            if !self.enabled {
                warn!("{} is disabled", self.title)
            }

            let tickers = self.get_tickers(&self.channel_type).await;
            
            if let Some(tickers) = tickers {
                self.reconnect(&tickers).await;
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<super::websocket::Ticker>) {
        if !self.enabled {
            return;
        }

        let chunk_size = 16;
        let batch_delay = 1.25;
        let semaphore = Arc::new(Semaphore::new(12));
        let rate_limiter = Arc::new(Semaphore::new(0));

        tokio::spawn({
            let rl = rate_limiter.clone();
            async move {
                // Formula: max limit / min secs limit = result; second in millis / result = min mill secs
                let mut interval = tokio::time::interval(Duration::from_millis(25)); // 40 rps 
                loop {
                    interval.tick().await;
                    rl.add_permits(1);
                }
            }
        });
        
        let notify = Arc::new(Notify::new());
        notify.notify_one();

        loop {
            notify.notified().await;
            println!("{}: Reconnecting...", self.title);

            let token = CancellationToken::new();
            let this = self.clone();
            
            for chunk in tickers.chunks(chunk_size) {
                let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCmd>(chunk_size);

                let this = this.clone();
                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    let symbol_cl = symbol.clone();

                    match cmd_tx.send(WsCmd::Subscribe(symbol.clone())).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!("{}: {}", this.title, e)
                        }
                    };

                    let this = this.clone();
                    let semaphore = semaphore.clone();
                    let rate_limiter = rate_limiter.clone();

                    tokio::spawn(async move {
                        let _permit = semaphore.clone().acquire_owned().await.unwrap();
                        let _rl = rate_limiter.acquire().await.unwrap();

                        let data = this.clone().get_ticker_snapshot_with_retry(&symbol_cl).await;

                        if let Some(json) = data {
                            let result = this.clone().handle_snapshot(json).await;
                            if let Some(event) = result {
                                match this.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", this.title, e)
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_secs_f64(batch_delay)).await; // Задержка перед следущим батчем
                    }); 
                }

                let token = token.clone();
                let this = this.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = token.cancelled() => {
                            return ;
                        }
                        _ = this.run_websocket(&mut cmd_rx) => {
                            token.cancel();
                        }
                    }
                });     
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<super::websocket::WsCmd>) -> super::websocket::WebSocketStatus {
        let url = Url::parse("wss://wbs-api.mexc.com/ws").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.title));
        let (mut write, mut read) = ws_stream.split();

        println!("🌐 {} is running", self.title);

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    if ticker == "SOLUSDT" {
                        println!("{}", ticker)
                    }
                    write.send(TungsteniteMessage::Text(
                        serde_json::json!({
                            "method": "SUBSCRIPTION",
                            "params": [
                                format!("spot@public.aggre.depth.v3.api.pb@100ms@{ticker}"),
                                format!("spot@public.miniTicker.v3.api.pb@{ticker}@UTC+8")
                            ]
                        }).to_string().into()
                    )).await.unwrap();
                }
            }
        }

        while let Some(msg) = read.next().await {
            let msg_type = match msg {
                Ok(m) => Some(m),
                Err(e) => {
                    tracing::error!("{}: {}", self.title, e);
                    None
                }
            };

            if let Some(msg_type) = msg_type {
                match msg_type {
                    TungsteniteMessage::Binary(binary) => {
                        let event = Event::decode(&*binary).unwrap();
                        
                        if event.channel.contains("depth") {
                            let json = OrderBookEvent::decode(&*binary).unwrap();
                            let result = self.clone().handle_delta(json).await;

                            if let Some(event) = result {
                                match self.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", self.title, e)
                                    }
                                }
                            }
                        }

                        if event.channel.contains("miniTicker") {
                            let data = TickerEvent::decode(&*binary).unwrap();
                            let json = TickerEventWithSymbol {
                                data: data,
                                symbol: Some(event.symbol)
                            };

                            let result = self.clone().handle_price(json).await;

                            if let Some(event) = result {
                                match self.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", self.title, e)
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

    async fn get_snapshot(self: std::sync::Arc<Self>, snapshot_tx: tokio::sync::mpsc::UnboundedSender<super::orderbook::SnapshotUi>) {
        if !self.enabled {
            return;
        }

        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(10);
            let this = Arc::clone(&self);

            loop {
                let ticker = ticker.clone();

                match this.sender_data.send(BookEvent::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("{}: {}", this.title, e)
                    }
                }

                tokio::select! {
                    data = rx.recv() => {
                        if let Some(snapshot_ui) = data {
                            if let Some(snapshot) = snapshot_ui {
                                match snapshot_tx.send(snapshot) {
                                    Ok(_) => {},
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<super::websocket::Ticker>> {
        let url = "https://api.mexc.com/api/v3/ticker/bookTicker";
        let response = self.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let Ok(tickers) = response.json::<Vec<Ticker>>().await else { return None };
        let usdt_tickers: Vec<Ticker> = tickers
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: std::sync::Arc<Self>, json: Self::Snapshot) -> Option<super::orderbook::BookEvent> {
        let Some(ticker) = json.symbol else { return None };
        let asks = parse_levels__(json.asks).await;
        let bids = parse_levels__(json.bids).await;
        let last_update_id = json.last_update_id;
        
        Some(BookEvent::Snapshot { 
            ticker: ticker.to_lowercase(),  
            snapshot: Snapshot { a: asks, b: bids, last_price: 0.0, last_update_id: Some(last_update_id) } 
        })
    }

    async fn handle_delta(self: std::sync::Arc<Self>, json: Self::Delta) -> Option<super::orderbook::BookEvent> {
        let Some(depths) = json.public_increase_depths else { return None };
        let asks_vec: Vec<Vec<String>> = depths.asks
            .into_iter()
            .map(|x| vec![x.price, x.quantity])
            .collect();

        let bids_vec: Vec<Vec<String>> = depths.bids
            .into_iter()
            .map(|x| vec![x.price, x.quantity])
            .collect();

        let asks = parse_levels__(asks_vec).await;
        let bids = parse_levels__(bids_vec).await;

        let ticker = json.symbol.to_lowercase();
        let from_version = depths.from_version.parse::<u64>().unwrap();
        let to_version = depths.to_version.parse::<u64>().unwrap();

        Some(BookEvent::Delta { 
            ticker: ticker.to_lowercase(), 
            delta: Delta { a: asks, b: bids, from_version: Some(from_version), to_version: Some(to_version)}
        }) 
    }

    async fn handle_price(self: std::sync::Arc<Self>, json: Self::Price) -> Option<super::orderbook::BookEvent> {
        let Some(deals) = json.data.public_deals else { return None };
        let Some(ticker) = json.symbol else { return None };
        let ticker = ticker.to_lowercase();
        let last_price = match deals.price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        Some(BookEvent::Price { ticker, last_price })
    }
}