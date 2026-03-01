use std::{fmt::{self, Display}};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::models::orderbook::MarketType;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Copy, Eq, Hash)]
#[serde(rename_all="snake_case")]
pub enum ExchangeType {
    Binance, 
    Bybit,
    #[serde(rename="kucoin")]
    KuCoin,
    BinX,
    Mexc,
    Gate,
    LBank,
    Unknown
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
    pub symbol: Option<String>,
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
    pub ticker: String,
    pub pair: String,
    pub spread: OrderedFloat<f64>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Eq, Serialize)]
pub struct ExchangePairs {
    pub long_pair: String,
    pub short_pair: String
}

impl ExchangePairs {
    pub fn new() -> Self {
        Self { 
            long_pair: String::new(), 
            short_pair: String::new() 
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (MarketType, &str)> {
        [
            (MarketType::Long, self.long_pair.as_str()),
            (MarketType::Short, self.short_pair.as_str())
        ]
        .into_iter()
    }
}