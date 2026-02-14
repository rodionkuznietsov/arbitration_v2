use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ExchangeType {
    Binance, 
    Bybit,
    KuCoin,
    BinX,
    Mexc,
    Gate,
    LBank,
    Unknown
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEventData {
    #[serde(rename="symbol", alias="currency_pair")]
    pub symbol: Option<String>,
    #[serde(rename="lastPrice", alias="last")]
    pub last_price: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    #[serde(rename="result", alias="data")]
    pub result: Option<TickerEventData>
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