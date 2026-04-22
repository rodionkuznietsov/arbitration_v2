use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::TickerInfo, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::ExchangeStoreCMD}};

#[allow(unused)]
pub struct BinanceAdapter;

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

    fn parse_orderbook(
        self: Arc<Self>,
        msg: Arc<String>
    ) {
        
    }
}