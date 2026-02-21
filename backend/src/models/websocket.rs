use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::models::{line::Line, exchange::ExchangeType, orderbook::{MarketType, SnapshotUi}};

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
    OrderBook(ChannelType, MarketType, SnapshotUi, String),
    LinesHistory(ChannelType, Vec<Line>, String, MarketType),
    UpdateLine(ChannelType, Line, MarketType),
    UpdateHistory(ChannelType, Line, MarketType),
}

#[derive(Display, Deserialize, Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case")]
pub enum ChannelType {
    OrderBook,
    LinesHistory,
    UpdateLine,
    UpdateHistory,
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