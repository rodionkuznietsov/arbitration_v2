use std::{sync::Arc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{data_mapping::{DataJson, SnapshotJson}, exchange::ExchangeType, websocket::{ChannelSubscription, ClientId, Symbol, WsClientMessage}};

pub enum ClientAggregatorUse {
    #[allow(unused)]
    UnRegister(ClientId),
    Subscribe(ClientId, ChannelSubscription),
    PublishJson(
        ChannelSubscription,
        WsClientMessage
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct KeyMarketType {
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub symbol: Arc<Symbol>,
}

impl KeyMarketType {
    pub fn new(
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        symbol: Arc<Symbol>
    ) -> Self {
        Self { 
            long_exchange, 
            short_exchange,
            symbol
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyPair {
    pub long_market_type: KeyMarketType,
    pub short_market_type: KeyMarketType,
}

impl KeyPair {
    #[allow(unused)]
    pub fn new(
        long_market_type: KeyMarketType,
        short_market_type: KeyMarketType,
    ) -> Self {
        Self { 
            long_market_type, 
            short_market_type, 
        }
    }
}

#[derive(Deserialize, Clone, Serialize, Debug, Hash, PartialEq, Eq)]
pub enum JsonPairUniqueId {
    OrderBook,
    LinesHistory,
    UpdateLine,
    Volume24h,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all="snake_case")]
pub enum JsonPairData {
    OrderBook {
        long: Arc<SnapshotJson>,
        short: Arc<SnapshotJson>,
    },
    LinesHistory {
        long: Vec<Value>,
        short: Vec<Value>,
    },
    UpdateLine {
        long: Value,
        short: Value,
    },
    Volume24h {
        long: Value,
        short: Value,
    },
}

impl Default for JsonPairData {
    fn default() -> Self {
        Self::LinesHistory { 
            long: Vec::new(), 
            short: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpreadPair {
    pub symbol: Arc<Symbol>,
    pub long_exchange: ExchangeType,
    pub long_spread: f64,
    pub short_exchange: ExchangeType,
    pub short_spread: f64,
    pub timestamp: i64
}

impl SpreadPair {
    pub fn new(
        symbol: Arc<Symbol>,
        long_exchange: ExchangeType,
        long_spread: f64,
        short_exchange: ExchangeType,
        short_spread: f64,
        timestamp: i64,
    ) -> Self {
        Self { 
            symbol,
            long_exchange,
            long_spread, 
            short_exchange,
            short_spread, 
            timestamp 
        }
    }
}

#[derive(Clone, Debug)]
pub struct Quote {
    pub exchange_id: Option<ExchangeType>,
    pub symbol: Option<Arc<Symbol>>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
}

impl Quote {
    pub fn new() -> Self {
        Self { 
            exchange_id: None, 
            symbol: None, 
            bid: None, 
            ask: None 
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Volume {
    #[serde(skip_serializing)]
    pub exchange_id: ExchangeType,
    
    pub value: Option<f64>,

    #[serde(skip_serializing)]
    pub symbol: Arc<Symbol>
}