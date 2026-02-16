use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, Semaphore, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

use crate::{models::{self, exchange::ExchangeType, orderbook::{BookEvent, Delta, Snapshot, SnapshotUi}, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::{market_manager::ExchangeWebsocket, orderbook_manager::OrderBookComand}};
use crate::services::{websocket::Websocket, orderbook_manager::{parse_levels__, OrderBookManager}};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="a")]
    asks: Vec<Vec<String>>,
    #[serde(rename="b")]
    bids: Vec<Vec<String>>,
    #[serde(rename="s")]
    symbol: String,
    #[serde(rename="U")]
    from_version: u64,
    #[serde(rename="u")]
    to_version: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    #[serde(rename="s")]
    symbol: String,
    #[serde(rename="c")]
    last_price: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SnapshotResponse {
    #[serde(rename="asks")]
    asks: Vec<Vec<String>>,
    #[serde(rename="bids")]
    bids: Vec<Vec<String>>,
    #[serde(rename="lastUpdateId")]
    last_update_id: u64,
    symbol: Option<String>
}

pub struct BinanceWebsocket {
    title: String,
    enabled: bool,
    channel_type: String,
    client: reqwest::Client,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    ticker_rx: async_channel::Receiver<(String, String)>,
    sender_data: mpsc::Sender<OrderBookComand>
}

impl BinanceWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = "[BinanceWebsocket]:".to_string();
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();
        let (ticker_tx, ticker_rx) = async_channel::bounded::<(String, String)>(1);
        let (sender_data, rx_data) = mpsc::channel::<OrderBookComand>(1);

        let book_manager = OrderBookManager::new(rx_data, ExchangeType::Binance);

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, channel_type,
            client, ticker_rx, ticker_tx,
            sender_data
        });

        let this_cl = this.clone();
        this_cl.connect();

        this
    }

    async fn get_ticker_snapshot_with_retry(self: Arc<Self>, ticker: &str) -> Option<SnapshotResponse> {
        let this = self.clone();
        let mut delay = Duration::from_secs(0);
        let max_delay = Duration::from_secs(60);
        let mut _snapshot = None;
        
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        loop {
            notify.notified().await;
            let result = this.clone().get_ticker_snapshot(ticker).await;
            if let Some((status, snapshot)) = result {
                if status == 429 {
                    delay = std::cmp::min(delay * 2, max_delay);
                    warn!("{} Reconnecting to ticker: {} in {} secs", this.title, ticker, delay.as_secs_f64());
                    tokio::time::sleep(delay).await;
                    notify.notify_one();
                } else if status == 418 {
                    error!("{} too many attempts. Reconneting in 10 minutes", this.title);
                    tokio::time::sleep(Duration::from_mins(10)).await;
                    notify.notify_one();
                } else {
                    _snapshot = Some(snapshot);
                    break;
                }
            }
        }

        _snapshot
    }

    async fn get_ticker_snapshot(self: Arc<Self>, ticker: &str) -> Option<(u16, SnapshotResponse)> {
        let url = format!("https://api.binance.com/api/v3/depth?symbol={}&limit=5000", ticker);
        let response = self.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let status = response.status().as_u16();
        let Ok(mut snapshot) = response.json::<SnapshotResponse>().await else { return None };
        snapshot.symbol = Some(ticker.to_string());
        Some((status, snapshot))
    }
}

impl Websocket for BinanceWebsocket {
    type Snapshot = SnapshotResponse;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        tokio::spawn(async move {
            if !self.enabled {
                warn!("{} is disabled", self.title);
                return;
            }

            let tickers = self.get_tickers(&self.channel_type).await;
            if let Some(tickers) = tickers {
                self.reconnect(&tickers).await;
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<models::websocket::Ticker>) {
        let chunk_size = 5;
        let batch_delay = 1.25;

        let semaphore = Arc::new(Semaphore::new(10));
        let rate_limiter = Arc::new(Semaphore::new(0));
        
        tokio::spawn({
            let rl = rate_limiter.clone();
            async move {
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
            info!("{}", format!("{} Reconnection...", self.title));

            let token = CancellationToken::new();

            for chunk in tickers.chunks(chunk_size) {
                let (cmd_tx, cmd_rx) = mpsc::channel::<WsCmd>(chunk_size);
                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    let symbol_cl = symbol.clone();
                    if cmd_tx.send(WsCmd::Subscribe(symbol)).await.is_err() {
                        error!("{} is it not possible to send the command", self.title)
                    }

                    tokio::spawn({
                        let semaphore = semaphore.clone();
                        let rate_limiter = rate_limiter.clone();
                        let this = self.clone();

                        async move {
                            let _permit = semaphore.acquire_owned().await.unwrap();
                            let _rl = rate_limiter.acquire().await.unwrap();
                            let this_cl = this.clone();

                            let result = this.get_ticker_snapshot_with_retry(&symbol_cl).await;
                            if let Some(snapshot) = result {
                                let this_cl_2 = this_cl.clone();
                                let result = this_cl.handle_snapshot(snapshot).await;
                                if let Some(cmd) = result {
                                    match this_cl_2.sender_data.send(cmd).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("{} Failed to send event: {e}", this_cl_2.title);
                                        }
                                    }
                                }
                            }

                            tokio::time::sleep(Duration::from_secs_f64(batch_delay)).await;
                        }
                    });
                }

                tokio::spawn({
                    let mut cmd_rx = cmd_rx;
                    let this = self.clone();
                    let token = token.clone();
                    let notify = notify.clone();

                    async move {
                        tokio::select! {
                            _ = token.cancelled() => {
                                return ;
                            }
                            _ = this.run_websocket(&mut cmd_rx) => {
                                token.cancel();
                            }
                        };
                        
                        tokio::time::sleep(Duration::from_secs(1)).await; 
                        notify.notify_one(); // Реконектемся
                    }
                });
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut tokio::sync::mpsc::Receiver<models::websocket::WsCmd>) -> models::websocket::WebSocketStatus {
        let url = Url::parse("wss://stream.binance.com:443/ws").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Binance] Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        info!("{}", format!("{} is now live", self.title));

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    info!("{}", ticker);

                    let orderbook = format!("{}@depth@100ms", ticker.to_lowercase());
                    let price = format!("{}@ticker", ticker.to_lowercase());

                    write.send(tokio_tungstenite::tungstenite::Message::Text(
                        serde_json::json!({
                            "method": "SUBSCRIBE",
                            "params": [
                                orderbook,
                                price
                            ]
                        }).to_string().into()
                    )).await.unwrap();
                }
            }
        }   

        let this = self.clone();
        while let Some(msg) = read.next().await {
            let msg = match msg {
                Ok(m) => Some(m),
                Err(e) => {
                    error!("{} {}", self.title, e);
                    None
                }
            };

            let this = this.clone();
            if let Some(msg_type) = msg {
                match msg_type {
                    Message::Text(channel) => {
                        if channel.contains("depthUpdate") {
                            let data: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                            let this_cl = this.clone();

                            let result = this.handle_delta(data).await;
                            if let Some(event) = result {
                                match this_cl.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!("{} Failed to send event: {e}", this_cl.title);
                                        break;
                                    }
                                }
                            }
                        } else if channel.contains("24hrTicker") {
                            let data: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let this_cl = this.clone();

                            let result = this.handle_price(data).await;
                            if let Some(event) = result {
                                match this_cl.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!("{} Failed to send event: {e}", this_cl.title);
                                        break;
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
        if !self.enabled {
            return;
        }
        
        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(10);
            let this = Arc::clone(&self);

            loop {
                let ticker = ticker.clone();

                match this.sender_data.send(OrderBookComand::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("{}: {}", this.title, e)
                    }
                }

                tokio::select! {
                    data = rx.recv() => {
                        if let Some(snapshot_ui) = data {
                            if let Some(snapshot) = snapshot_ui {
                                match snapshot_tx.send(snapshot).await {
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

    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<models::websocket::Ticker>> {
        let url = "https://api.binance.com/api/v3/ticker/bookTicker";
        let response = self.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let Ok(tickers) = response.json::<Vec<Ticker>>().await else { return None };
        let usdt_tickers: Vec<Ticker> = tickers
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();
        
        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<OrderBookComand> {
        let Some(ticker) = json.symbol else { return None };
        let ticker = ticker.to_lowercase();
        let asks = parse_levels__(json.asks).await;
        let bids = parse_levels__(json.bids).await;
        let last_update_id = json.last_update_id;

        Some(OrderBookComand::Event(
            BookEvent::Snapshot {
                    ticker,
                    snapshot: Snapshot {
                    a: asks,
                    b: bids,
                    last_price: 0.0,
                    last_update_id: Some(last_update_id)
                }
            }
        ))
    }

    async fn handle_delta(self: Arc<Self>, json: Self::Delta) -> Option<OrderBookComand> {
        let ticker = json.symbol.to_lowercase();
        let asks = parse_levels__(json.asks).await;
        let bids = parse_levels__(json.bids).await;
        let from_version = json.from_version;
        let to_version = json.to_version;

        Some(OrderBookComand::Event(
            BookEvent::Delta { 
                ticker, 
                delta: Delta {
                    a: asks,
                    b: bids,
                    from_version: Some(from_version),
                    to_version: Some(to_version)
                }
            }
        ))
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<OrderBookComand> {
        let ticker = json.symbol.to_lowercase();
        let last_price = match json.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };
        
        Some(OrderBookComand::Event(
            BookEvent::Price { 
                ticker, 
                last_price 
            }
        ))
    }
}

#[async_trait]
impl ExchangeWebsocket for BinanceWebsocket {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)> {
        self.ticker_tx.clone()
    }

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>) {
        self.get_last_snapshot(snapshot_tx).await
    }

    async fn get_spread(
        self: Arc<Self>, 
        spread_tx: mpsc::Sender<Option<(ExchangeType, Option<f64>, Option<f64>)>>
    ) {
        self.sender_data.send(OrderBookComand::GetBestAskAndBidPrice { 
            ticker: "btc".to_string(),
            reply: spread_tx
        }).await.unwrap();
    }
}