use std::{sync::{Arc}};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::{models::{exchange::{ExchangeType, TickerEvent}, orderbook::OrderBookEvent, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::{aggregator::AggregatorCommand, exchange_setup::{ExchangeSetup}, exchange_aggregator::ExchangeStoreCMD}};
use crate::models::orderbook::{BookEvent, Delta, Snapshot};
use crate::services::{websocket::Websocket, exchange_aggregator::{parse_levels__}};

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

#[derive(Clone)]
pub struct BybitWebsocket {
    setup: Arc<ExchangeSetup>,
}

impl BybitWebsocket {
    pub fn new(
        enabled: bool,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    ) -> Arc<Self> {
        let setup = ExchangeSetup::new(
            ExchangeType::Bybit, 
            enabled,
            aggregator_tx
        );

        let this = Arc::new(
            Self { 
                setup,
            }
        );

        this.clone().connect();
        this
    }

    async fn handle_volume24hr(self: Arc<Self>, json: TickerEvent) -> Option<ExchangeStoreCMD> {
        let Some(result) = json.result else { return None };
        let Some(ticker) = result.symbol else { return None };
        let Some(volume24hr) = result.volume else { return None };
        let volume = match volume24hr.parse::<f64>() {
            Ok(v) => v,
            Err(_) => 0.0
        };
        
        Some(ExchangeStoreCMD::Event(BookEvent::Volume24hr { ticker: ticker.to_lowercase(), volume }))
    }
}

impl Websocket for BybitWebsocket {
    type Snapshot = OrderBookEvent;
    type Delta = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>) {
        tokio::spawn({
            async move {
                if !self.setup.enabled {
                    warn!("{} is disabled", self.setup.title);
                    return;
                } 

                let tickers = self.get_tickers(&self.setup.channel_type).await;
                if let Some(tickers) = tickers {
                    self.reconnect(&tickers).await;
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
            tracing::info!("{}", format!("{} Reconnection...", self.setup.title));

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
                            tracing::info!("{}", format!("{} Failed to send subscribe command: {}", self.setup.title, e));
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus {
        let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
        let (ws_stream, _) = connect_async(url.to_string()).await.expect(&format!("{} Failed to connect", self.setup.title));
        let (mut write, mut read) = ws_stream.split();

        tracing::info!("{}", format!("{} is now live", self.setup.title));

        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Subscribe(ticker) => {

                    let orderbook_str = format!("orderbook.50.{}USDT", ticker.to_uppercase());
                    let price_str = format!("tickers.{}USDT", ticker.to_uppercase());

                    write.send(Message::Text(
                        serde_json::json!({
                            "op": "subscribe",
                            "channel_type": self.setup.channel_type,
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
                    error!("{} {}", self.setup.title, e);
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
                                    
                                    if let Some(cmd) = result {
                                        match this.setup.sender_data.send(cmd).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("{}: {}", this.setup.title, e);
                                                break;
                                            }
                                        }
                                    }
                                },
                                Some("delta") => {
                                    let result = this.clone().handle_delta(json).await;

                                    if let Some(event) = result {
                                        match this.setup.sender_data.send(event).await {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!("{}: {}", this.setup.title, e);
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
                            let price_result = this.clone().handle_price(json.clone()).await;
                            if let Some(event) = price_result {
                                match this.setup.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", this.setup.title, e);
                                        break;
                                    }
                                }
                            }

                            let volume_result = this.clone().handle_volume24hr(json).await;
                            if let Some(event) = volume_result {
                                match this.setup.sender_data.send(event).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        tracing::error!("{}: {}", this.setup.title, e);
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

    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<Ticker>> {
        let url = format!("https://api.bybit.com/v5/market/tickers?category={channel_type}");
        let response = self.setup.client.get(url).send().await;

        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        let usdt_tickers = json.result.list
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: OrderBookEvent) -> Option<ExchangeStoreCMD> {
        let Some(data) = json.data else { return None };
        let Some(ticker) = data.symbol else { return None };
        let Some(asks) = data.asks else { return None };
        let Some(bids) = data.bids else { return None };

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        let ticker = ticker.to_lowercase();
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

    async fn handle_delta(self: Arc<Self>, json: OrderBookEvent) -> Option<ExchangeStoreCMD> {
        let Some(data) = json.data else { return None };
        let Some(ticker) = data.symbol else { return None };
        let Some(asks) = data.asks else { return None };
        let Some(bids) = data.bids else { return None };

        let asks = parse_levels__(asks).await;
        let bids = parse_levels__(bids).await;

        let ticker = ticker.to_lowercase();
        Some(ExchangeStoreCMD::Event(
            BookEvent::Delta { 
                ticker, 
                delta: Delta { 
                    a: asks, 
                    b: bids, 
                    from_version: None, 
                    to_version: None
                } 
            }
        ))
    }

    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<ExchangeStoreCMD> {
       let Some(data) = json.result else { return None };
        let Some(ticker) = data.symbol else { return None };
        let Some(last_price) = data.last_price else { return None };
        let last_price = match last_price.parse::<f64>() {
            Ok(p) => p,
            Err(_) => 0.0
        };

        let ticker = ticker.to_lowercase();

        Some(ExchangeStoreCMD::Event(
            BookEvent::Price { 
                ticker, 
                last_price 
            }
        ))
    }
}
