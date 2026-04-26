use std::{collections::{BTreeMap}};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::models::websocket::Symbol;

#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookEvent<'a> {
    #[serde(rename="cts", alias="time_ms")]
    pub timestamp: Option<i64>,
    #[serde(rename="type")]
    pub order_type: Option<&'a str>,
    #[serde(rename="data", alias="result")]
    pub data: Option<OrderBookEventData<'a>>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OrderBookEventData<'a> {
    #[serde(rename="s")]
    pub symbol: Option<&'a str>,
    #[serde(rename="a", alias="asks")]
    pub asks: Option<Vec<Vec<&'a str>>>,
    #[serde(rename="b", alias="bids")]
    pub bids: Option<Vec<Vec<&'a str>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: BTreeMap<Decimal, f64>,
    pub b: BTreeMap<Decimal, f64>,
    pub last_update_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Delta {
    pub a: BTreeMap<Decimal, f64>,
    pub b: BTreeMap<Decimal, f64>,
    pub from_version: Option<u64>,
    pub to_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotUi {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}

#[derive(Debug, Clone)]
pub enum BookEvent {
    Snapshot { 
        symbol: Symbol,
        snapshot: Snapshot,
    },
    Delta { 
        symbol: Symbol, 
        delta: Delta 
    },
    TickerUpdate {
        symbol: Symbol,
        last_price: f64,
        volume: f64,
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(bound(deserialize="'de: 'a"))]
pub struct OrderBookFromHttp<'a> {
    pub asks: Vec<Vec<&'a str>>,
    pub bids: Vec<Vec<&'a str>>,
    pub current: i64,
    pub update: i64
}