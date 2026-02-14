use serde::Deserialize;

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