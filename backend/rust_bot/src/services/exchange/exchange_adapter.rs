use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::TickerInfo, websocket::Symbol}, services::exchange::exchange_aggregator::ExchangeStoreCMD};

#[async_trait::async_trait]
pub trait ExchangeAdapter: Send + Sync + 'static {
    fn ws_url(self: Arc<Self>,) -> &'static str;
    fn requires_auth(self: Arc<Self>) -> bool;
    async fn auth_url(self: Arc<Self>, client: &reqwest::Client) -> Option<url::Url>;
    async fn get_api_key(self: Arc<Self>, client: &reqwest::Client) -> Result<String, reqwest::Error>;
    async fn get_tickers(self: Arc<Self>, client: &reqwest::Client) -> Option<Vec<TickerInfo>>;
    async fn get_snapshot_spot_http(self: Arc<Self>, tickers: &Vec<TickerInfo>, client: &reqwest::Client, sender_data: watch::Sender<ExchangeStoreCMD>);
    async fn parse_message(self: Arc<Self>, msg: String, snapshot_channel: mpsc::Sender<ExchangeStoreCMD>, sender_data: watch::Sender<ExchangeStoreCMD>);
    async fn parse_tickers(self: Arc<Self>, msg: Arc<String>, sender_data: watch::Sender<ExchangeStoreCMD>);
    fn parse_orderbook(self: Arc<Self>, msg: Arc<String>);
    fn create_subscribe_messages(self: Arc<Self>, symbol: Arc<Symbol>) -> Vec<Message>;
}