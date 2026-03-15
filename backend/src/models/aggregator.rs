use std::{sync::Arc};
use uuid::Uuid;
use crate::models::{exchange::ExchangeType, orderbook::SnapshotUi, websocket::{ChannelSubscription, ChartEvent, Symbol}};

pub enum ClientAggregatorUse {
    #[allow(unused)]
    UnRegister(Uuid),
    Subscribe(Uuid, ChannelSubscription),
    Publish {
        key: ChannelSubscription,
        payload: Arc<AggregatorPayload>
    }
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
pub struct KeyPair {
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub symbol: Arc<Symbol>
}

impl KeyPair {
    pub fn new(
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        symbol: Arc<Symbol>,
    ) -> Self {
        Self { 
            long_exchange, 
            short_exchange, 
            symbol 
        }
    }
}