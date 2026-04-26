use std::{collections::{BTreeMap}, sync::Arc};
use rust_decimal::Decimal;
use tokio::sync::{Mutex, mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{PriceCache, TickerInfo}, orderbook::OrderBookEventData, websocket::Symbol}, services::exchange::exchange_aggregator::ExchangeStoreCMD};

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
    async fn parse_orderbook(self: Arc<Self>, msg: Arc<String>, snapshot_channel: mpsc::Sender<ExchangeStoreCMD>, sender_data: watch::Sender<ExchangeStoreCMD>);
    async fn handle_snapshot<'a>(self: Arc<Self>, data: Option<OrderBookEventData<'a>>, snapshot_channel: mpsc::Sender<ExchangeStoreCMD>, sender_data: watch::Sender<ExchangeStoreCMD>);
    async fn handle_delta<'a>(self: Arc<Self>, data: Option<OrderBookEventData<'a>>, sender_data: watch::Sender<ExchangeStoreCMD>);
    fn cache(&self) -> &Arc<Mutex<PriceCache>>;
    async fn is_valid_price(self: Arc<Self>, last_price: f64, symbol: &Symbol) -> bool {        
        let mut price_cache = self.cache().lock().await;

        if let Some(old_price) = price_cache.exists_price(symbol.to_string()) {
            if *old_price != last_price {
                price_cache.add(symbol.clone(), last_price);
                return true;
            }
            return false;
        } else {
            price_cache.add(symbol.to_string(), last_price);
            return true
        }
    }
    fn is_valid_book(self: Arc<Self>, asks: &BTreeMap<Decimal, f64>, bids: &BTreeMap<Decimal, f64>) -> bool {
        asks.len() > 1 && bids.len() > 1
    }
    fn create_subscribe_messages(self: Arc<Self>, symbol: Arc<Symbol>) -> Vec<Message>;
}