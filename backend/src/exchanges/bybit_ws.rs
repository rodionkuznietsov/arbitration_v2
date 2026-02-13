use std::{collections::{HashMap}, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use tokio_util::sync::CancellationToken;

use crate::exchanges::{orderbook::{BookEvent, Delta, OrderBookManager, Snapshot, SnapshotUi, parse_levels__}, websocket::{Ticker, WebSocketStatus, Websocket, WsCmd}};

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
pub struct OrderBookEvent {
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
pub struct TickerEvent {
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

#[derive(Clone)]
pub struct BybitWebsocket {
    title: String,
    enabled: bool,
    client: reqwest::Client,
    channel_type: String,
    sender_data: mpsc::Sender<BookEvent>,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    ticker_rx: async_channel::Receiver<(String, String)>
}

impl BybitWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = "[Bybit-Websocket]".to_string();
        let client = reqwest::Client::new();
        let channel_type = String::from("spot");
        let (sender_data, rx_data) = mpsc::channel(10);
        let (ticker_tx, ticker_rx) = async_channel::bounded::<(String, String)>(1);

        let book_manager = OrderBookManager::new(rx_data);

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self { 
            enabled, channel_type, title, 
            client, sender_data, ticker_tx, 
            ticker_rx
        });

        let this_cl = Arc::clone(&this);
        this_cl.connect();
        this
    }
}

impl Websocket for BybitWebsocket {
    type Snapshot = OrderBookEvent;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        let this = Arc::clone(&self);
        tokio::spawn({
            async move {
                if !self.enabled {
                    println!("{} enabled: false", this.title);
                    return;
                } 

                let tickers = this.get_tickers(&this.channel_type).await;

                if let Some(tickers) = tickers {
                    this.reconnect(&tickers).await;
                }
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>) {
        let chunk_size = 5;
        let notify = Arc::new(Notify::new());

        notify.notify_one(); 
        loop {
            notify.notified().await;
            tracing::info!("{}", format!("{} Reconnection...", self.title));

            let token = CancellationToken::new();
            let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCmd>(10);

            let this = self.clone();
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

            for chunk in tickers.chunks(chunk_size) {
                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap().replace("USDT", "");
                    match cmd_tx.send(WsCmd::Subscribe(symbol)).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::info!("{}", format!("{} Failed to send subscribe command: {}", self.title, e));
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus {
        let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.title));
        let (mut write, mut read) = ws_stream.split();

        tracing::info!("{}", format!("{} is now live", self.title));

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {

                    let orderbook_str = format!("orderbook.50.{}USDT", ticker.to_uppercase());
                    let price_str = format!("tickers.{}USDT", ticker.to_uppercase());

                    write.send(Message::Text(
                        serde_json::json!({
                            "op": "subscribe",
                            "channel_type": self.channel_type,
                            "args": [
                                orderbook_str,
                                price_str
                            ]
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
                    println!("{}: {e}", this.title);
                    None 
                }
            };

            if let Some(msg_type) = msg_type {
                match msg_type {
                    Message::Text(channel) => {
                        if channel.contains("orderbook.") {
                            let json: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                            
                            match json.order_type.as_deref() {
                                Some("snapshot") => {
                                    let result = this.clone().handle_snapshot(json).await;

                                    if let Some(event) = result {
                                        match this.sender_data.send(event).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("{}: {}", this.title, e);
                                                break;
                                            }
                                        }
                                    }
                                },
                                Some("delta") => {
                                    let result = this.clone().handle_delta(json).await;

                                    if let Some(event) = result {
                                        match this.sender_data.send(event).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("{}: {}", this.title, e);
                                                break;
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }

                        if channel.contains("tickers.") {
                            let json: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let result = this.clone().handle_price(json).await;
                            if let Some(event) = result {
                                match this.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", this.title, e);
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

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::UnboundedSender<SnapshotUi>) {
        if !self.enabled {
            return ;
        }

        while let Ok((_uuid, ticker)) = self.ticker_rx.recv().await {
            let (tx, mut rx) = mpsc::channel(50);
            let this = self.clone();

            loop {
                let ticker = ticker.clone();

                match this.sender_data.send(BookEvent::GetBook { ticker, reply: tx.clone() }).await {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("{}: {}", this.title, e);
                        break;
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

    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<Ticker>> {
        let url = format!("https://api.bybit.com/v5/market/tickers?category={channel_type}");
        let response = self.client.get(url).send().await;

        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers = json.result.list
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: OrderBookEvent) -> Option<BookEvent> {
        let Some(data) = json.data else { return None };
        let Some(ticker) = data.symbol else { return None };
        let Some(asks) = data.asks else { return None };
        let Some(bids) = data.bids else { return None };

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        let ticker = ticker.to_lowercase();
        Some(BookEvent::Snapshot { 
            ticker, 
            snapshot: Snapshot { a: asks, b: bids, last_price: 0.0, last_update_id: None } 
        })
    }

    async fn handle_delta(self: Arc<Self>, json: OrderBookEvent) -> Option<BookEvent> {
        let Some(data) = json.data else { return None };
        let Some(ticker) = data.symbol else { return None };
        let Some(asks) = data.asks else { return None };
        let Some(bids) = data.bids else { return None };

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        let ticker = ticker.to_lowercase();
        Some(BookEvent::Delta { ticker, delta: Delta { a: asks, b: bids, from_version: None, to_version: None} })
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<BookEvent> {
       let Some(data) = json.data else { return None };
        let ticker = data.symbol;
        let last_price = match data.last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        let ticker = ticker.to_lowercase();

        Some(BookEvent::Price { ticker, last_price })
    }
}
