use serde::{Deserialize, Serialize};

use crate::models::exchange::ExchangeType;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Ticker {
    #[serde(rename="symbol", alias="id")]
    pub symbol: Option<String>
}

pub enum WsCmd {
    Subscribe(String)
}

#[derive(Debug, PartialEq)]
pub enum WebSocketStatus {
    Finished,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Subscription {
    pub action: String,
    pub channel: String,
    pub long_exchange: Option<ExchangeType>, 
    pub short_exchange: Option<ExchangeType>,
    pub ticker: String
}