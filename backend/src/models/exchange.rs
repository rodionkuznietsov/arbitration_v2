use std::sync::Arc;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::models::{websocket::Symbol};

#[derive(Display, Debug, Clone, PartialEq, Deserialize, Serialize, Copy, Eq, Hash, PartialOrd)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="PascalCase")]
#[repr(u8)]
pub enum ExchangeType {
    Binance = 0, 
    Bybit = 1,
    #[serde(rename="kucoin")]
    KuCoin = 2,
    BinX = 3,
    Mexc = 4,
    Gate = 5,
    LBank = 6,
    Unknown = 7
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TickerEventData {
    #[serde(rename="symbol", alias="currency_pair")]
    pub symbol: Option<Symbol>,
    #[serde(rename="lastPrice", alias="last")]
    pub last_price: Option<String>,
    #[serde(rename="turnover24h", alias="quote_volume")]
    pub volume: Option<String>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TickerEvent {
    #[serde(rename="result", alias="data")]
    pub result: Option<TickerEventData>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub symbol: Arc<Symbol>,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub spread: f64,
    pub timestamp: i64,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TickerResponse {
    #[serde(rename="result", alias="data")]
    pub result: TickerResult
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TickerResult {
    #[serde(rename="list", alias="ticker")]
    pub list: Vec<TickerInfo>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct TickerInfo {
    #[serde(rename="symbol", alias="id")]
    pub symbol: Option<String>
}