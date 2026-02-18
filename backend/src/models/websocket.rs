use serde::{Deserialize, Serialize};

use crate::models::{candle::Candle, exchange::ExchangeType, orderbook::{OrderType, SnapshotUi}};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Ticker {
    #[serde(rename="symbol", alias="id")]
    pub symbol: Option<String>
}

pub enum WsCmd {
    Subscribe(String)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="snake_case")]
pub enum ClientCmd {
    Subscribe,
    UnSubscribe
}

#[derive(Debug, Clone)]
pub enum ServerToClientEvent {
    OrderBook(ChannelType, OrderType, SnapshotUi, String),
    CandlesHistory(ChannelType, Vec<Candle>, String),
    UpdateCandle(ChannelType, Candle, String)
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all="snake_case")]
pub enum ChannelType {
    OrderBook,
    CandlesHistory,
    UpdateCandle
}

#[derive(Debug, PartialEq)]
pub enum WebSocketStatus {
    Finished,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Subscription {
    pub action: ClientCmd,
    pub channel: ChannelType,
    pub long_exchange: Option<ExchangeType>, 
    pub short_exchange: Option<ExchangeType>,
    pub ticker: String
}