use std::sync::Arc;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct SnapshotJson {
    pub asks: Vec<Value>,
    pub bids: Vec<Value>,
    pub last_price: OrderedFloat<f64>,
}

#[derive(Deserialize, Serialize, Debug,  Hash, PartialEq, Eq)]
pub enum DataJson {
    Snapshot(Arc<SnapshotJson>),
    LinesHistory(Arc<Vec<Value>>),
    UpdateLine(Value),
    Volume24h(Value)
}