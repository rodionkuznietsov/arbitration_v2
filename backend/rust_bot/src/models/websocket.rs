use std::{collections::{HashMap, VecDeque}, sync::Arc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::models::{aggregator::{JsonPairData, JsonPairUniqueId, KeyMarketType}, exchange::ExchangeType, line::Line};

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

#[derive(Display, Debug, Clone)]
#[strum(serialize_all="snake_case")]
pub enum ChartEvent {
    UpdateLine(f64, f64, i64),
    Volume24hr(f64, f64),
    LinesHistory(Arc<VecDeque<Line>>, Arc<VecDeque<Line>>),
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
        long_market_type: KeyMarketType,
        short_market_type: KeyMarketType,
    },
    Chart {
        long_market_type: KeyMarketType,
        short_market_type: KeyMarketType,
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

#[derive(Debug, PartialEq, Eq)]
pub struct ClientData {
    pub result: HashMap<JsonPairUniqueId, Arc<WsClientMessage>>
}

impl ClientData {
    pub fn new() -> Self {
        Self {
            result: HashMap::new()
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct WsClientMessage {
    pub channel: ChannelType,
    pub result: WsClientMsgResult,
}

impl Default for WsClientMessage {
    fn default() -> Self {
        Self { 
            channel: ChannelType::Unknown,
            result: WsClientMsgResult { 
                data: Arc::new(JsonPairData::default()), 
                symbol: Arc::new(Symbol::new()),
                unique_id: JsonPairUniqueId::Unknown
            } 
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct WsClientMsgResult {
    pub data: Arc<JsonPairData>,
    pub symbol: Arc<Symbol>,
    pub unique_id: JsonPairUniqueId,
}

impl Default for WsClientMsgResult {
    fn default() -> Self {
        Self { 
            data: Arc::new(JsonPairData::default()), 
            symbol: Arc::new(Symbol::new()),
            unique_id: JsonPairUniqueId::Unknown
        }
    }
}