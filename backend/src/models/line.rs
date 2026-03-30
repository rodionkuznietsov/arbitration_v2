use get_size::GetSize;
use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow};

use crate::models::{exchange::ExchangeType, websocket::Symbol};

#[derive(Debug, Clone, FromRow, GetSize)]
pub struct Line {
    pub timestamp: i64,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub value: f64
}

impl Line {
    pub fn new(
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        symbol: Symbol,
        value: f64,
        timeframe: TimeFrame,
        timestamp: i64,
    ) -> Self {
        Self { 
            long_exchange,
            short_exchange, 
            symbol, 
            value,
            timeframe, 
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type, GetSize)]
#[sqlx(type_name="timeframe", rename_all="lowercase")]
pub enum TimeFrame {
    #[serde(rename="1m")]
    #[sqlx(rename="1m")]
    One
}