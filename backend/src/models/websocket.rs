use std::{collections::VecDeque, sync::Arc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::models::{exchange::ExchangeType, line::{Line}, orderbook::SnapshotUi};

pub type ClientId = Uuid;
pub type Symbol = String;

pub enum WsCmd {
    Subscribe(Vec<Message>)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="snake_case")]
pub enum ClientCmd {
    Subscribe,
    UnSubscribe
}

#[derive(Display, Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="snake_case")]
pub enum ChartEvent {
    UpdateLine(f64, f64, i64),
    Volume24hr(f64, f64),
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Subscription {
    pub action: ClientCmd,
    pub channel: ChannelType,
    pub long_exchange: Option<ExchangeType>, 
    pub short_exchange: Option<ExchangeType>,
    pub ticker: Symbol
}

#[derive(Debug)]
pub struct ChartData {
    pub ticker: Option<Arc<Symbol>>,
    pub volume24h: Option<(f64, f64)>,
    pub update_line: Option<(f64, f64, i64)>,
    pub long_lines: Option<VecDeque<Line>>,
    pub short_lines: Option<VecDeque<Line>>,
}

impl ChartData {
    pub fn new() -> Self {
        Self {
            ticker: None,
            volume24h: None, 
            update_line: None,
            long_lines: None,
            short_lines: None,
        }
    }
}

pub struct OrderBookData {
    pub ticker: Option<Arc<Symbol>>,
    pub long: Option<Arc<SnapshotUi>>,
    pub short: Option<Arc<SnapshotUi>>,
}

impl OrderBookData {
    pub fn new() -> Self {
        Self {
            ticker: None,
            long: None, 
            short: None
        }
    }
}