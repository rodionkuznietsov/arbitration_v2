use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::{exchanges::exchange_adapter::ExchangeAdapter, models::{exchange::{ExchangeType, TickerInfo}, websocket::Symbol}, services::{exchange_aggregator::ExchangeStoreCMD}};

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
        client: &reqwest::Client
    ) -> Option<url::Url> {
        todo!()
    }

    async fn get_api_key(
        self: Arc<Self>,
        client: &reqwest::Client
    ) -> Result<String, reqwest::Error> {
        todo!()
    }

    async fn get_tickers(self: Arc<Self>, client: &reqwest::Client) -> Option<Vec<TickerInfo>> {
        None
    }
    
    fn create_subscribe_messages(
        self: Arc<Self>,
        symbol: Arc<Symbol>
    ) -> Vec<Message> {
        todo!()
    }

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        data_aggregator_tx: mpsc::Sender<ExchangeStoreCMD>
    ) {
        
    }
}