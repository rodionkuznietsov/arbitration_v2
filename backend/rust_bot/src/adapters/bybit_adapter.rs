use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{TickerEvent, TickerInfo, TickerResponse}, orderbook::{BookEvent, Delta, OrderBookEvent, Snapshot}, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::{ExchangeStoreCMD, parse_levels__}}};

pub struct BybitAdapter;

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

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>,
    ) {
        if msg.contains("orderbook") {
            let json: OrderBookEvent = serde_json::from_str(&msg).unwrap();
                let data = json.data;
                let ts = json.timestamp;

                match json.order_type.as_deref() {
                    Some("snapshot") => {
                        if let Some(data) = data {
                            let ticker = data.symbol;
                            let asks = data.asks;
                            let bids = data.bids;

                            if let (
                                Some(symbol), 
                                Some(asks), 
                                Some(bids),
                                Some(timestamp)
                            ) = (ticker, asks, bids, ts) {
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
                                                timestamp,
                                            }
                                        }
                                    )
                                ).await;
                            }
                        }
                    },
                    Some("delta") => {
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
                    },
                    _ => {}
                }
        }

        if msg.contains("tickers") {
            let json: TickerEvent = serde_json::from_str(&msg).unwrap();
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
                    let last_price = price_str.parse().expect("BybitAdapter -> Не удалось преобразовать price_str в f64");
                    let volume = vol_str.parse().expect("BybitAdapter -> Не удалось преобразовать vol_str в f64");

                    sender_data.send(
                        ExchangeStoreCMD::Event(
                            BookEvent::TickerUpdate { 
                                symbol, 
                                last_price, 
                                volume 
                            }
                        )
                    ).ok();
                    
                }
            }
        }
    }
}