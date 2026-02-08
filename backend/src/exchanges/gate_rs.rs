use std::{collections::HashMap, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

use crate::exchanges::{orderbook::{BookEvent, OrderBookManager, Snapshot, parse_levels__}, websocket::{Ticker, WebSocketStatus, Websocket, WsCmd}};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="result")]
    result: OrderBookEventData
}

#[derive(Debug, Deserialize, Serialize)]
struct OrderBookEventData {
    #[serde(rename="s", alias="s")]
    symbol: Option<String>,
    #[serde(rename="bids")]
    bids: Option<Vec<Vec<String>>>,
    #[serde(rename="asks")]
    asks: Option<Vec<Vec<String>>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    result: TickerEventData
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEventData {
    #[serde(rename="currency_pair")]
    symbol: Option<String>,
    #[serde(rename="last")]
    last_price: Option<String>
}

pub struct GateWebsocket {
    title: String,
    enabled: bool,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    ticker_rx: async_channel::Receiver<(String, String)>,
    channel_type: String,
    client: reqwest::Client,
    sender_data: mpsc::Sender<BookEvent>,
}

impl GateWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let title = String::from("[GateWebsocket]");
        let (ticker_tx, ticker_rx) = async_channel::bounded(1);
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<BookEvent>(10);

        let book_manager = OrderBookManager {
            books: HashMap::new(),
            rx: rx_data
        };

        tokio::spawn(async move {
            book_manager.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, channel_type,
            ticker_tx, ticker_rx, client,
            sender_data
        });

        let this_cl = this.clone();
        this_cl.connect();

        this
    }
}

impl Websocket for GateWebsocket {
    type Snapshot = OrderBookEvent;
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

    async fn reconnect(self: Arc<Self>, tickers: &Vec<super::websocket::Ticker>) {
        let chunk_size = 50;
        let reconnect_delay = 500; // ms
        
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        loop {
            notify.notified().await;
            info!("{}", format!("{} Reconnection...", self.title));

            let token = CancellationToken::new();

            for chunk in tickers.chunks(chunk_size) {
                let (cmd_tx, cmd_rx) = mpsc::channel(chunk_size); 

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    if let Some(err) = cmd_tx.send(WsCmd::Subscribe(symbol)).await.err() {
                        error!("{} {}", self.title, err);
                        return ;
                    }
                }

                tokio::spawn({
                    let token = token.clone();
                    let mut cmd_rx = cmd_rx;
                    let notify = notify.clone();
                    let this = self.clone();

                    async move {
                        tokio::select! {
                            _ = token.cancelled() => {
                                return ;
                            }
                            _ = this.run_websocket(&mut cmd_rx) => {
                                token.cancel();
                            }
                        }

                        tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
                        notify.notify_one();
                    }
                });
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut tokio::sync::mpsc::Receiver<super::websocket::WsCmd>) -> super::websocket::WebSocketStatus {
        let url = Url::parse("wss://api.gateio.ws/ws/v4/").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        info!("{}", format!("{} is now live", self.title));

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
                    // println!("{}", ticker);
                    write.send(Message::Text(
                        serde_json::json!({
                            "channel": "spot.order_book",
                            "event": "subscribe",
                            "payload": [ticker, "50", "100ms"]
                        }).to_string().into()
                    )).await.unwrap();

                    write.send(Message::Text(
                        serde_json::json!({
                            "channel": "spot.tickers",
                            "event": "subscribe",
                            "payload": [ticker]
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

            if let Some(msg_type) = msg {
                match msg_type {
                    Message::Text(channel) => {
                        if channel.contains("spot.order_book") {
                            let data: OrderBookEvent = serde_json::from_str(&channel).unwrap();
                            let this_cl = this.clone();
                            let result = this.clone().handle_snapshot(data).await;
                            if let Some(event) = result {
                                if let Some(err) = this_cl.sender_data.send(event).await.err() {
                                    error!("{} {}", this_cl.title, err);
                                    break;
                                }
                            }
                        } else if channel.contains("spot.tickers") {
                            let data: TickerEvent = serde_json::from_str(&channel).unwrap();
                            let this_cl = this.clone();
                            let result = this.clone().handle_price(data).await;
                            if let Some(event) = result {
                                if let Some(err) = this_cl.sender_data.send(event).await.err() {
                                    error!("{} {}", this_cl.title, err);
                                    break;
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

    async fn get_snapshot(self: Arc<Self>, snapshot_tx: tokio::sync::mpsc::UnboundedSender<super::orderbook::SnapshotUi>) {
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
        let url = "https://api.gateio.ws/api/v4/spot/currency_pairs";
        let response = self.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let Ok(tickers) = response.json::<Vec<Ticker>>().await else { return None };
        let usdt_tickers: Vec<Ticker> = tickers
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<super::orderbook::BookEvent> {
        let Some(ticker) = json.result.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();

        let Some(asks) = json.result.asks else { return None };
        let Some(bids) = json.result.bids else { return None };

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        Some(BookEvent::Snapshot { ticker: ticker, snapshot: Snapshot {
            a: asks,
            b: bids,
            last_price: 0.0,
            last_update_id: None
        }})
    }

    async fn handle_delta(self: Arc<Self>, _json: Self::Delta) -> Option<super::orderbook::BookEvent> {
        todo!()
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<super::orderbook::BookEvent> {
        let Some(ticker) = json.result.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();
        let Some(last_price_str) = json.result.last_price else { return None };
        let last_price = match last_price_str.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };
        
        Some(BookEvent::Price { ticker, last_price })
    }
}