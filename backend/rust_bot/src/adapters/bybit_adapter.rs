use std::{sync::Arc};
use tokio::sync::{Mutex, mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{PriceCache, TickerEvent, TickerInfo, TickerResponse}, orderbook::{BookEvent, Delta, OrderBookEvent, OrderBookEventData, Snapshot}, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::{ExchangeStoreCMD, parse_levels__}}};

pub struct BybitAdapter {
    price_cache: Arc<Mutex<PriceCache>>
}

impl BybitAdapter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { price_cache: Arc::new(Mutex::new(PriceCache::new())) })
    }
}

#[async_trait::async_trait]
impl ExchangeAdapter for BybitAdapter {
    fn ws_url(self: Arc<Self>) -> &'static str {
        "wss://stream.bybit.com/v5/public/spot"
    }

    fn requires_auth(
        self: Arc<Self>
    ) -> bool {
        false
    }

    async fn auth_url(
        self: Arc<Self>,
        _client: &reqwest::Client
    ) -> Option<url::Url> {
        todo!()
    }

    async fn get_api_key(
        self: Arc<Self>,
        _client: &reqwest::Client
    ) -> Result<String, reqwest::Error> {
        todo!()
    }

    async fn get_tickers(self: Arc<Self>, client: &reqwest::Client) -> Option<Vec<TickerInfo>> {
        let url = format!("https://api.bybit.com/v5/market/tickers?category=spot");
        let response = client.get(url).send().await;

        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };
        
        let usdt_tickers: Vec<TickerInfo> = json.result.list
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)        
    }

    async fn get_snapshot_spot_http(
        self: Arc<Self>,
        _tickers: &Vec<TickerInfo>,
        _client: &reqwest::Client,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        
    }

    fn create_subscribe_messages(
        self: Arc<Self>,
        symbol: Arc<Symbol>
    ) -> Vec<Message> {
        let orderbook_str = format!("orderbook.50.{}", symbol);
        let price_str = format!("tickers.{}", symbol);

        let message = Message::Text(
            serde_json::json!({
                "op": "subscribe",
                "channel_type": "spot",
                "args": [
                    orderbook_str,
                    price_str
                ]
            }).to_string().into()
        );
        
        vec![message]
    }

    fn cache(
        &self,
    ) -> &Arc<Mutex<PriceCache>> {
        &self.price_cache
    }

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>,
    ) {
        let msg_arc = Arc::new(msg);

        if msg_arc.contains("orderbook") {
            self.clone().parse_orderbook(msg_arc.clone(), snapshot_channel, sender_data.clone()).await;
        }

        if msg_arc.contains("tickers") {
            self.parse_tickers(msg_arc, sender_data.clone()).await;
        }
    }

    async fn parse_tickers(
        self: Arc<Self>,
        msg: Arc<String>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        let json: TickerEvent<'_> = serde_json::from_str(&msg).unwrap();
        let result = json.result;
        
        if let Some(data) = result {
            if let (
                Some(symbol),
                Some(price_str), 
                Some(vol_str)
            ) = (
                data.symbol, 
                data.last_price, 
                data.volume
            ) {
                let symbol = symbol.to_lowercase();
                let last_price = price_str.parse::<f64>().expect("BybitAdapter -> Не удалось преобразовать price_str в f64");
                let volume = vol_str.parse::<f64>().expect("BybitAdapter -> Не удалось преобразовать vol_str в f64");

                let is_new_price = self.is_valid_price(last_price, &symbol).await;
                if is_new_price {
                    let _ = sender_data.send(
                        ExchangeStoreCMD::Event(
                            BookEvent::TickerUpdate { 
                                symbol, 
                                last_price, 
                                volume 
                            }
                        )
                    );
                }
            }
        }
    }

    async fn parse_orderbook(
        self: Arc<Self>,
        msg: Arc<String>,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        
        let json: OrderBookEvent<'_> = serde_json::from_str(&msg).unwrap();
        let data = json.data;

        match json.order_type.as_deref() {
            Some("snapshot") => {
                self.handle_snapshot(data, snapshot_channel, sender_data.clone()).await;
            },
            Some("delta") => {
                self.handle_delta(data, sender_data).await;
            },
            _ => {}
        }
    }

    async fn handle_snapshot<'a>(
        self: Arc<Self>,
        data: Option<OrderBookEventData<'a>>,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        if let Some(data) = data {
            let ticker = data.symbol;
            let asks = data.asks;
            let bids = data.bids;

            if let (
                Some(symbol), 
                Some(asks), 
                Some(bids),
            ) = (ticker, asks, bids) {
                let symbol = symbol.to_lowercase();
                let asks = parse_levels__(asks);
                let bids = parse_levels__(bids);

                let _ = snapshot_channel.send(
                    ExchangeStoreCMD::Event(
                        BookEvent::Snapshot { 
                            symbol: symbol, 
                            snapshot: Snapshot {
                                a: asks,
                                b: bids,
                                last_update_id: None,
                            }
                        }
                    )
                ).await;
            }
        }
    }

    async fn handle_delta<'a>(
        self: Arc<Self>,
        data: Option<OrderBookEventData<'a>>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        if let Some(data) = data {
            let ticker = data.symbol;
            let asks = data.asks;
            let bids = data.bids;

            if let (
                Some(symbol), 
                Some(asks), 
                Some(bids
            )) = (ticker, asks, bids) {
                let symbol = symbol.to_lowercase();
                let asks = parse_levels__(asks);
                let bids = parse_levels__(bids);
                let is_valid = self.is_valid_book(&asks, &bids);
                if is_valid {
                    let _ = sender_data.send(
                        ExchangeStoreCMD::Event(
                            BookEvent::Delta { 
                                symbol: symbol, 
                                delta: Delta {
                                    a: asks,
                                    b: bids,
                                    from_version: None,
                                    to_version: None
                                }
                            }
                        )
                    );
                }
            }
        }
    }
}