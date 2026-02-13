use std::{sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc};

use crate::models::orderbook::{BookEvent, SnapshotUi};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Ticker {
    #[serde(rename="symbol", alias="id")]
    pub symbol: Option<String>
}

pub enum WsCmd {
    Subscribe(String)
}

#[derive(Debug, PartialEq)]
pub enum WebSocketStatus {
    Finished,
}

pub trait Websocket {
    type Snapshot;
    type Delta;
    type Price;

    fn connect(self: Arc<Self>);
    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>);
    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus;
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>);
    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<Ticker>>;
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<BookEvent>;
    async fn handle_delta(self: Arc<Self>, json: Self::Delta) -> Option<BookEvent>;
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<BookEvent>;
}