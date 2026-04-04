use std::{sync::Arc, time::{Duration}};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

use crate::{models::{self, exchange::ExchangeType, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::{data_aggregator::AggregatorCommand, exchange_setup::ExchangeSetup, exchange_aggregator::ExchangeStoreCMD}};
use crate::models::orderbook::{BookEvent, Snapshot};
use crate::services::{websocket::Websocket, exchange_aggregator::{parse_levels__}};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="depth")]
    depth: OrderBookEventData,
    #[serde(rename="pair")]
    symbol: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OrderBookEventData {
    #[serde(rename="bids")]
    bids: Vec<Vec<String>>,
    #[serde(rename="asks")]
    asks: Vec<Vec<String>>
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerResponse {
    data: Vec<Ticker>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    tick: TickerEventData,
    #[serde(rename="pair")]
    symbol: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEventData {
    #[serde(rename="latest")]
    last_price: f64
}

pub struct LBankWebsocket {
    setup: Arc<ExchangeSetup>,
}

impl LBankWebsocket {
    pub fn new(
        enabled: bool,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    ) -> Arc<Self> {
        let setup = ExchangeSetup::new(
            ExchangeType::LBank, 
            enabled,
            aggregator_tx.clone()
        );
        
        let this = Arc::new(
            Self { 
                setup,
            }
        );

        this.clone().connect();
        this
    }
}

impl Websocket for LBankWebsocket {
    type Snapshot = OrderBookEvent;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        tokio::spawn(async move {
            if !self.setup.enabled {
                warn!("{} is disabled", self.setup.title);
                return;
            }

            let tickers = self.get_tickers(&self.setup.channel_type).await;
            if let Some(tickers) = tickers {
                self.reconnect(&tickers).await;
            }
        });
    }

    async fn reconnect(self: Arc<Self>, tickers: &Vec<models::websocket::Ticker>) {
        // let (cmd_tx, cmd_rx) = mpsc::channel(1); 

        // cmd_tx.send(WsCmd::Subscribe("btc_usdt".to_string())).await.unwrap();

        // tokio::spawn({
        //     let mut cmd_rx = cmd_rx;
        //     let this = self.clone();

        //     async move {
        //         this.run_websocket(&mut cmd_rx).await;
        //     }
        // });
        let chunk_size = 1;
        let reconnect_delay = 500; // ms
        
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        loop {
            notify.notified().await;
            info!("{}", format!("{} Reconnection...", self.setup.title));

            let token = CancellationToken::new();

            for chunk in tickers[0..1].chunks(chunk_size) {
                let (cmd_tx, cmd_rx) = mpsc::channel(chunk_size); 

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    // if let Some(err) = cmd_tx.send(WsCmd::Subscribe(symbol)).await.err() {
                    //     error!("{} {}", self.setup.title, err);
                    //     return ;
                    // }
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
                            _ = this.run_websocket(&mut cmd_rx, None) => {
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

    async fn run_websocket(
        self: Arc<Self>, 
        cmd_rx: &mut tokio::sync::mpsc::Receiver<models::websocket::WsCmd>,
        _api_token: Option<String>
    ) -> models::websocket::WebSocketStatus {
        let url = Url::parse("wss://www.lbkex.net/ws/V2/").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.unwrap();
        let (mut write, _read) = ws_stream.split();

        info!("{}", format!("{} is now live", self.setup.title));

        let (tx, mut rx) = mpsc::unbounded_channel();

        let write_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            } 
        });

        // while let Some(cmd) = cmd_rx.recv().await {
        //     match cmd {
        //         WsCmd::Subscribe(ticker) => {
        //             println!("{}", ticker);
        //             let depth_sub = serde_json::json!({
        //                 "action": "subscribe",
        //                 "subscribe": "depth",
        //                 "depth": "10",
        //                 "pair": ticker
        //             });
                    
        //             if tx.send(Message::Text(depth_sub.to_string())).is_err() {
        //                 println!("Error sending depth subscription");
        //                 break;
        //             }

        //             let ticker_sub = serde_json::json!({
        //                 "action": "subscribe",
        //                 "subscribe": "tick",
        //                 "pair": ticker
        //             });
                    
        //             if tx.send(Message::Text(ticker_sub.to_string())).is_err() {
        //                 break;
        //             }
        //         }
        //     }
        // }

        write_handle.abort();
        WebSocketStatus::Finished
    }

    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<models::websocket::Ticker>> {
        let url = "https://api.lbkex.com/v2/accuracy.do";
        let response = self.setup.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let Ok(tickers) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers: Vec<Ticker> = tickers.data
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("usdt"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<ExchangeStoreCMD> {
        let Some(ticker) = json.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();

        let asks = json.depth.asks;
        let bids = json.depth.bids;

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        Some(ExchangeStoreCMD::Event(
            BookEvent::Snapshot { 
                ticker: ticker, 
                snapshot: Snapshot {
                    a: asks,
                    b: bids,
                    last_price: 0.0,
                    last_update_id: None
                }
            }
        ))
    }

    async fn handle_delta(self: Arc<Self>, _json: Self::Delta) -> Option<ExchangeStoreCMD> {
        todo!()
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<ExchangeStoreCMD> {
        let Some(ticker) = json.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();
        
        Some(ExchangeStoreCMD::Event(
            BookEvent::Price { 
                ticker, 
                last_price: json.tick.last_price 
            }
        ))
    }
}
