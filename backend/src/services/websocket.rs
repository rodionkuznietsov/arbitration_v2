use std::{sync::Arc};
use tokio::sync::{mpsc};
use crate::{models::{orderbook::SnapshotUi, websocket::{Ticker, WebSocketStatus, WsCmd}}, services::orderbook_manager::OrderBookComand};

pub trait Websocket {
    type Snapshot;
    type Delta;
    type Price;

    fn connect(self: Arc<Self>);
    async fn reconnect(self: Arc<Self>, tickers: &Vec<Ticker>);
    async fn run_websocket(self: Arc<Self>, cmd_rx: &mut mpsc::Receiver<WsCmd>) -> WebSocketStatus;
    async fn get_last_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>);
    async fn get_tickers(&self, channel_type: &str) -> Option<Vec<Ticker>>;
    async fn handle_snapshot(self: Arc<Self>, json: Self::Snapshot) -> Option<OrderBookComand>;
    async fn handle_delta(self: Arc<Self>, json: Self::Delta) -> Option<OrderBookComand>;
    async fn handle_price(self: Arc<Self>, json: Self::Price) -> Option<OrderBookComand>;
}