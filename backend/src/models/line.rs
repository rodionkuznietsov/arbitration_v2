use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow};

use crate::models::{websocket::Symbol};

#[derive(Debug, Clone, FromRow)]
pub struct Line {
    pub timestamp: i64,
    pub exchange_pair: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub value: f64
}

impl Line {
    pub fn new(
        exchange_pair: String,
        symbol: Symbol,
        value: f64,
        timeframe: TimeFrame,
        timestamp: i64,
    ) -> Self {
        Self { 
            exchange_pair, 
            symbol, 
            value,
            timeframe, 
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[sqlx(type_name="timeframe", rename_all="lowercase")]
pub enum TimeFrame {
    #[serde(rename="1m")]
    #[sqlx(rename="1m")]
    One
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
pub enum MarketType {
    Long,
    Short
}