use std::sync::Arc;

use crate::models::{orderbook::SnapshotUi, websocket::ChartEvent};

#[derive(Clone, Debug)]
pub struct AggregatorMessage {
    pub long_value: Option<f64>,
    pub short_value: Option<f64>,
    pub long_order_book: Option<Arc<SnapshotUi>>,
    pub short_order_book: Option<Arc<SnapshotUi>>
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub enum AggregatorEvent {
    ChartEvent(ChartEvent),
    OrderBook
}