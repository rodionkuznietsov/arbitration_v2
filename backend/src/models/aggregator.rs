use std::{collections::VecDeque, sync::Arc};
use crate::models::{exchange::ExchangeType, line::Line, orderbook::SnapshotUi, websocket::{ChannelSubscription, ChartEvent, ClientId, Symbol}};

pub enum ClientAggregatorUse {
    #[allow(unused)]
    UnRegister(ClientId),
    Subscribe(ClientId, ChannelSubscription),
    Publish {
        key: ChannelSubscription,
        payload: Arc<AggregatorPayload>
    },
}

#[derive(Clone, Debug)]
pub enum AggregatorPayload {
    OrderBook {
        long_order_book: Arc<SnapshotUi>,
        short_order_book: Arc<SnapshotUi>,
        ticker: Arc<Symbol>,
    },
    #[allow(unused)]
    ChartEvent {
        event: ChartEvent,
        ticker: Arc<Symbol>
    }
}

impl Default for AggregatorPayload {
    fn default() -> Self {
        Self::OrderBook { 
            long_order_book: Arc::new(
                SnapshotUi { 
                    a: Vec::new(), 
                    b: Vec::new(), 
                    last_price: 0.0, 
                    timestamp: 0 
                }
            ), 
            short_order_book: Arc::new(
                SnapshotUi { 
                    a: Vec::new(), 
                    b: Vec::new(), 
                    last_price: 0.0, 
                    timestamp: 0 
                }
            ), 
            ticker: Arc::new(String::from("")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug)]
pub struct SpreadPair {
    pub long_spread: f64,
    pub short_spread: f64,
    pub timestamp: i64
}

impl SpreadPair {
    pub fn new(
        long_spread: f64,
        short_spread: f64,
        timestamp: i64,
    ) -> Self {
        Self { 
            long_spread, 
            short_spread, 
            timestamp 
        }
    }
}

