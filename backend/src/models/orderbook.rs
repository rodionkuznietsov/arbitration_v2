use std::{collections::{BTreeMap}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Hash)]
pub enum OrderType {
    Long, 
    Short
}

#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="type")]
    pub order_type: Option<String>,
    #[serde(rename="data", alias="result")]
    pub data: Option<OrderBookEventData>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OrderBookEventData {
    #[serde(rename="s")]
    pub symbol: Option<String>,
    #[serde(rename="a", alias="asks")]
    pub asks: Option<Vec<Vec<String>>>,
    #[serde(rename="b", alias="bids")]
    pub bids: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub last_price: f64,
    pub last_update_id: Option<u64>
}

#[derive(Debug, Clone, Serialize)]
pub struct Delta {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub from_version: Option<u64>,
    pub to_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotUi {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}

#[derive(Debug)]
pub enum BookEvent {
    Snapshot { 
        ticker: String, 
        snapshot: Snapshot,
    },
    Delta { 
        ticker: String, 
        delta: Delta 
    },
    Price { 
        ticker: String, 
        last_price: f64 
    }
}