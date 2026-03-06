use std::sync::Arc;
use uuid::Uuid;
use crate::models::{orderbook::SnapshotUi, websocket::{ChannelSubscription, ChartEvent, Symbol}};

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
        ticker: Arc<Symbol>
    },
    #[allow(unused)]
    ChartEvent {
        event: ChartEvent,
        ticker: Arc<Symbol>
    }
}