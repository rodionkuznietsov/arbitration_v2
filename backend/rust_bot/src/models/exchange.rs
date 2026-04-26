use std::collections::HashMap;

use get_size::GetSize;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::Display;

use crate::models::websocket::Symbol;

#[derive(Display, Debug, Clone, Type, PartialEq, Deserialize, Serialize, Copy, Eq, Hash, PartialOrd, GetSize)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="PascalCase")]
#[sqlx(type_name="exchange_type")]
pub enum ExchangeType {
    Binance, 
    Bybit,
    #[serde(rename="kucoin")]
    KuCoin,
    BinX,
    Mexc,
    #[serde(rename="gate.io")]
    Gate,
    LBank,
    Unknown
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TickerEventData<'a> {
    #[serde(rename="symbol", alias="currency_pair")]
    pub symbol: Option<&'a str>,
    #[serde(rename="lastPrice", alias="last")]
    pub last_price: Option<&'a str>,
    #[serde(rename="turnover24h", alias="quote_volume")]
    pub volume: Option<&'a str>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TickerEvent<'a> {
    #[serde(rename="result", alias="data")]
    pub result: Option<TickerEventData<'a>>
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

#[derive(Debug, Clone)]
pub struct PriceCache {
    pub old_prices: HashMap<String, f64>
}

impl PriceCache {
    pub fn new() -> Self {
        Self { old_prices: HashMap::new() }
    }

    pub fn exists_price(
        &self,
        symbol: Symbol
    ) -> Option<&f64> {
        self.old_prices.get(&symbol)
    }

    pub fn add(
        &mut self,
        symbol: Symbol, 
        last_price: f64
    ) {
        self.old_prices.insert(symbol, last_price);
    }
}