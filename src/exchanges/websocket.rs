use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use anyhow::Result;

use crate::exchanges::orderbook::LocalOrderBook;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Ticker {
    #[serde(rename="symbol")]
    pub symbol: Option<String>
}

pub trait Websocket {
    type Snapshot;
    type Price;

    fn connect(self: Arc<Self>, channel_type: String);
    fn get_local_book(self: Arc<Self>) -> Arc<RwLock<LocalOrderBook>>;
    async fn get_tickers(&self, channel_type: &str) -> Result<Vec<Ticker>>;
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<(String, BTreeMap<i64, f64>, BTreeMap<i64, f64>)>;
    async fn handle_delta(self: Arc<Self>, json: Self::Snapshot);
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<(String, f64)>;
}