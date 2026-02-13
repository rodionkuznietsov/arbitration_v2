use std::collections::HashMap;

use crate::models::candle::Candle;
use crate::exchanges::orderbook::{OrderType, SnapshotUi};
use crate::services::market_manager::ExchangeType;

#[derive(Debug, Clone)]
pub struct UserState {
    pub candle_history: Vec<Candle>,
    pub ticker: String,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub orderbooks: HashMap<String, SnapshotUi>,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            candle_history: Vec::new(),
            ticker: String::new(),
            long_exchange: ExchangeType::Unknown,
            short_exchange: ExchangeType::Unknown,
            orderbooks: HashMap::new(),
        }
    }

    pub async fn update_book(&mut self, order_type: OrderType, snapshot: SnapshotUi) {
        let key = match order_type {
            OrderType::Long => "long".to_string(),
            OrderType::Short => "short".to_string(),
        };
        self.orderbooks.insert(key, snapshot);
    }

    pub async fn add_candles(&mut self, candles: Vec<Candle>) {
        for candle in candles {
            self.candle_history.push(candle);
        }
    }

    pub async fn update_exchange_info(
        &mut self, 
        ticker: &str, 
        long_exchange: ExchangeType, 
        short_exchange: ExchangeType
    ) {
        self.ticker = ticker.to_string();
        self.long_exchange = long_exchange;
        self.short_exchange = short_exchange;
    }
}