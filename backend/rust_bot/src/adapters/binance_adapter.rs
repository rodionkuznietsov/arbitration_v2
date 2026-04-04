use std::sync::Arc;

use tokio::sync::mpsc;
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
    
    fn create_subscribe_messages(
        self: Arc<Self>,
        _symbol: Arc<Symbol>
    ) -> Vec<Message> {
        todo!()
    }

    async fn parse_message(
        self: Arc<Self>,
        _msg: String,
        _data_aggregator_tx: mpsc::Sender<ExchangeStoreCMD>
    ) {
        
    }
}