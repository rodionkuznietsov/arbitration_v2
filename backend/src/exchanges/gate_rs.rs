use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{Notify, broadcast, mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use url::Url;

use crate::{exchanges::exchange_setup::ExchangeSetup, models::{self, exchange::{ExchangeType, TickerEvent}, orderbook::{OrderBookEvent, SnapshotUi}, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::{aggregator::AggregatorCommand, exchange_store::ExchangeStoreCMD, market_manager::ExchangeWebsocket}};
use crate::models::orderbook::{BookEvent, Snapshot};
use crate::services::{websocket::Websocket, exchange_store::{parse_levels__}};

pub struct GateWebsocket {
    setup: Arc<ExchangeSetup>,
    aggregator_tx: mpsc::Sender<AggregatorCommand>
}

impl GateWebsocket {
    pub fn new(
        enabled: bool, 
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    ) -> Arc<Self> {
        let setup = ExchangeSetup::new(ExchangeType::Gate, enabled);
        let this = Arc::new(
            Self { 
                setup,
                aggregator_tx: aggregator_tx.clone()
            }
        );

        this.clone().connect();
        this.setup.clone().spawn_quote_updater(aggregator_tx);
        this.clone().spawn_oderbooks_updater();

        this
    }

    fn spawn_oderbooks_updater(self: Arc<Self>) {
        let mut rx = self.setup.books_updates.subscribe();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        let (reply, rx) = oneshot::channel::<SnapshotUi>();
                        self.setup.sender_data.send(ExchangeStoreCMD::GetBook { ticker: ticker.clone(), reply }).await.ok();

                        if let Ok(snapshot) = rx.await {
                            self.aggregator_tx.send(AggregatorCommand::UpdateOrderbooks { 
                                exchange_type: ExchangeType::Gate,
                                snapshot_ui: snapshot,
                                ticker,
                            }).await.ok();
                        }   
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    _ => continue
                }
            }
        });
    }

    async fn handle_volume24hr(self: Arc<Self>, json: TickerEvent) -> Option<ExchangeStoreCMD> {
        let Some(result) = json.result else { return None };
        let Some(ticker) = result.symbol else { return None };
        let Some(volume24hr) = result.volume else { return None };
        let volume = match volume24hr.parse::<f64>() {
            Ok(v) => v,
            Err(_) => 0.0
        };

        let ticker = ticker.replace("_", "");
        
        Some(ExchangeStoreCMD::Event(BookEvent::Volume24hr { ticker: ticker.to_lowercase(), volume }))
    }

    pub async fn get_volume24hr(
        self: Arc<Self>,
        volume_tx: broadcast::Sender<(ExchangeType, String, f64)>
    ) {
        if self.setup.sender_data.send(ExchangeStoreCMD::GetVolume24hr { ticker: "moca".to_string(), reply: volume_tx }).await.is_err() {
            return ;
        }
    }
}

impl Websocket for GateWebsocket {
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
        let chunk_size = 50;
        let reconnect_delay = 500; // ms
        
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        loop {
            notify.notified().await;
            info!("{}", format!("{} Reconnection...", self.setup.title));

            let token = CancellationToken::new();

            for chunk in tickers.chunks(chunk_size) {
                let (cmd_tx, cmd_rx) = mpsc::channel(chunk_size); 

                for ticker in chunk {
                    let symbol = ticker.symbol.clone().unwrap();
                    if let Some(err) = cmd_tx.send(WsCmd::Subscribe(symbol)).await.err() {
                        error!("{} {}", self.setup.title, err);
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

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut tokio::sync::mpsc::Receiver<models::websocket::WsCmd>) -> models::websocket::WebSocketStatus {
        let url = Url::parse("wss://api.gateio.ws/ws/v4/").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.unwrap();
        let (mut write, mut read) = ws_stream.split();

        info!("{}", format!("{} is now live", self.setup.title));

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {
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
                    error!("{} {}", self.setup.title, e);
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
                                if let Some(err) = this_cl.setup.sender_data.send(event).await.err() {
                                    error!("{} {}", this_cl.setup.title, err);
                                    break;
                                }
                            }
                        } else if channel.contains("spot.tickers") {
                            let data: TickerEvent = serde_json::from_str(&channel).unwrap();

                            let price_result = this.clone().handle_price(data.clone()).await;
                            if let Some(event) = price_result {
                                if let Some(err) = this.clone().setup.sender_data.send(event).await.err() {
                                    error!("{} {}", this.setup.title, err);
                                    break;
                                }
                            }
                            
                            let volume24hr_result = this.clone().handle_volume24hr(data).await;
                            if let Some(event) = volume24hr_result {
                                if let Some(err) = this.setup.sender_data.send(event).await.err() {
                                    error!("{} {}", this.setup.title, err);
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

    async fn get_last_snapshot(self: Arc<Self>, snapshot_tx: tokio::sync::mpsc::Sender<SnapshotUi>) {
        if !self.setup.enabled {
            return;
        }
        
        while let Ok((_uuid, ticker)) = self.setup.ticker_rx.recv().await {
            let this = Arc::clone(&self);

            loop {
                let (tx, mut rx) = oneshot::channel();
                let ticker = ticker.clone();
                match this.setup.sender_data.send(ExchangeStoreCMD::GetBook { ticker, reply: tx }).await {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("{}: {}", this.setup.title, e)
                    }
                }

                tokio::select! {
                    data = rx => {
                        if let Ok(snapshot) = data {
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

    async fn get_tickers(&self, _channel_type: &str) -> Option<Vec<models::websocket::Ticker>> {
        let url = "https://api.gateio.ws/api/v4/spot/currency_pairs";
        let response = self.setup.client.get(url).send().await;
        let Ok(response) = response else { return None };
        let Ok(tickers) = response.json::<Vec<Ticker>>().await else { return None };
        let usdt_tickers: Vec<Ticker> = tickers
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<ExchangeStoreCMD> {
        let Some(data) = json.data else { return None };
        let Some(ticker) = data.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();

        let Some(asks) = data.asks else { return None };
        let Some(bids) = data.bids else { return None };

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
        let Some(data) = json.result else { return None };
        let Some(ticker) = data.symbol else { return None };
        let ticker = ticker.replace("_", "").to_lowercase();
        let Some(last_price_str) = data.last_price else { return None };
        let last_price = match last_price_str.parse::<f64>() {
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

// #[async_trait]
// impl ExchangeWebsocket for GateWebsocket {
//     fn ticker_tx(&self) -> async_channel::Sender<(String, String)> {
//         self.setup.ticker_tx.clone()
//     }

//     async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>) {
//         self.get_last_snapshot(snapshot_tx).await
//     }

//     fn spawn_quote_updater(
//         self: Arc<Self>
//     ) {
//         let mut rx = self.setup.books_updates.subscribe();
//         let title = self.setup.title.clone();
        
//         tokio::spawn(async move {
//             loop {
//                 match rx.recv().await {
//                     Ok(ticker) => {
//                         let (reply, rx) = oneshot::channel::<(f64, f64)>();
//                         if self.setup.sender_data.send(ExchangeStoreCMD::Quote { ticker: ticker.clone(), reply: reply }).await.is_err() {
//                             continue;
//                         }
                        
//                         if let Ok((ask, bid)) = rx.await {
//                             if self.aggregator_tx.clone().send(
//                                 AggregatorCommand::UpdateQuotes { 
//                                     exchange_type: ExchangeType::Gate,
//                                     ticker: ticker.clone(),
//                                     ask,
//                                     bid
//                                 }
//                             ).await.is_err() {
//                                 continue;
//                             }
//                         }
//                     },
//                     Err(broadcast::error::RecvError::Lagged(_)) => {
//                         continue;
//                     },
//                     Err(broadcast::error::RecvError::Closed) => {
//                         warn!("{} Канал спреда закрыт", title);
//                         break;
//                     }
//                 }
//             }
//         });
//     }
// }