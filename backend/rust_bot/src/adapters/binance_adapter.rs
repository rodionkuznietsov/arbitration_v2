use std::{collections::BTreeMap, sync::Arc};

use rust_decimal::Decimal;
use tokio::sync::{Mutex, mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{PriceCache, TickerInfo}, orderbook::OrderBookEventData, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::ExchangeStoreCMD}};

#[allow(unused)]
pub struct BinanceAdapter {
    price_cache: Arc<Mutex<PriceCache>>
}

impl BinanceAdapter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { price_cache: Arc::new(Mutex::new(PriceCache::new())) })
    }
}

#[async_trait::async_trait]
impl ExchangeAdapter for BinanceAdapter {
    fn ws_url(self: Arc<Self>) -> &'static str {
        "wss://stream.binance.com:443/ws"
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

    async fn get_tickers(self: Arc<Self>, _client: &reqwest::Client) -> Option<Vec<TickerInfo>> {
        None
    }
    
    async fn get_snapshot_spot_http(
        self: Arc<Self>,
        tickers: &Vec<TickerInfo>,
        client: &reqwest::Client,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        
    }

    fn create_subscribe_messages(
        self: Arc<Self>,
        _symbol: Arc<Symbol>
    ) -> Vec<Message> {
        todo!()
    }

    fn cache(
        &self,
    ) -> &Arc<Mutex<PriceCache>> {
        &self.price_cache
    }

    async fn parse_message(
        self: Arc<Self>,
        _msg: String,
        _snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        _data_aggregator_tx: watch::Sender<ExchangeStoreCMD>,
    ) {
        
    }

    async fn parse_tickers(
        self: Arc<Self>,
        msg: Arc<String>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {

    }

    async fn is_valid_price(    
        self: Arc<Self>,
        last_price: f64,
        symbol: &Symbol
    ) -> bool {
        true
    }

    async fn parse_orderbook(
        self: Arc<Self>,
        _msg: Arc<String>,
        _snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        
    }

    async fn handle_snapshot<'a>(
        self: Arc<Self>,
        _data: Option<OrderBookEventData<'a>>,
        _snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {

    }

    async fn handle_delta<'a>(
        self: Arc<Self>,
        _data: Option<OrderBookEventData<'a>>,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {

    }
}