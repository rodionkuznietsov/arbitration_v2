use get_size::GetSize;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum_macros::Display;

use crate::models::{websocket::Symbol};

#[derive(Display, Debug, Clone, Type, PartialEq, Deserialize, Serialize, Copy, Eq, Hash, PartialOrd, GetSize)]
#[strum(serialize_all="PascalCase")]
#[sqlx(type_name="exchange_type")]
pub enum ExchangeType {
    Binance, 
    #[serde(rename="bybit")]
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