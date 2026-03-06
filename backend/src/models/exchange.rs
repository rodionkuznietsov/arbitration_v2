use std::{fmt::{self, Display}};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::models::{websocket::Symbol};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Copy, Eq, Hash, PartialOrd)]
#[serde(rename_all="snake_case")]
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

impl Display for ExchangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ExchangeType::Binance => "binance",
            ExchangeType::Bybit => "bybit",
            ExchangeType::BinX => "binx",
            ExchangeType::Gate => "gate",
            ExchangeType::KuCoin => "kucoin",
            ExchangeType::LBank => "lbank",
            ExchangeType::Mexc => "mexc",
            ExchangeType::Unknown => "unknown"
        };
        write!(f, "{}", s)
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spread {
    pub ticker: Symbol,
    pub pair: String,
    pub spread: OrderedFloat<f64>
}