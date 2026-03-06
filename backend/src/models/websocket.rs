use std::{sync::Arc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use uuid::Uuid;

use crate::models::{exchange::{ ExchangeType}, orderbook::{SnapshotUi}};

pub type ClientId = Uuid;
pub type Symbol = String;

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

#[derive(Display, Deserialize, Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case")]
pub enum ChartEvent {
    UpdateLine,
    Volume24hr,
    LinesHistory,
    UpdateHistory
}

#[derive(Display, Deserialize, Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case")]
pub enum ChannelType {
    OrderBook,
    Chart,
    Unknown
}

#[derive(Display, Deserialize, Debug, Clone, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case")]
pub enum ChannelSubscription {
    OrderBook {
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        ticker: Arc<Symbol>,
    },
    Chart {
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        ticker: Arc<Symbol>
    }
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
    pub ticker: Symbol
}

#[derive(Serialize)]
#[serde(rename_all="snake_case")]
pub enum WebsocketResult {
    #[serde(rename="books")]
    OrderBook {
        long: Arc<SnapshotUi>,
        short: Arc<SnapshotUi>
    },
    #[allow(unused)]
    LinesHistory {
        value: String,
        time: u64,
    },
    #[allow(unused)]
    Unknown
}

#[derive(Serialize)]
pub struct WsMessage {
    pub channel: ChannelType,
    pub result: WebsocketResult,
    pub ticker: Arc<Symbol>
}