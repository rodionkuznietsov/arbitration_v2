use std::{sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc};

use crate::exchanges::orderbook::{BookEvent, SnapshotUi};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Ticker {
    #[serde(rename="symbol")]
    pub symbol: Option<String>
}

pub trait Websocket {
    type Snapshot;
    type Price;

    fn connect(self: Arc<Self>);
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::UnboundedSender<SnapshotUi>);
    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<Ticker>>;
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<BookEvent>;
    async fn handle_delta(self: Arc<Self>, json: Self::Snapshot);
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<BookEvent>;
}