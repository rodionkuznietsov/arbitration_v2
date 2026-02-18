use std::str::FromStr;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow, types::BigDecimal};

#[derive(Debug, Clone, FromRow)]
pub struct Candle {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange_pair: String,
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
}

impl Candle {
    pub fn new() -> Self {
        Self { 
            timestamp: Utc::now(), 
            exchange_pair: String::new(), 
            symbol: String::new(), 
            timeframe: TimeFrame::Five, 
            open: BigDecimal::from_str("0.0").unwrap(), 
            high: BigDecimal::from_str("0.0").unwrap(), 
            low: BigDecimal::from_str("0.0").unwrap(), 
            close: BigDecimal::from_str("0.0").unwrap()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[sqlx(type_name="timeframe")]
pub enum TimeFrame {
    #[serde(rename="5m")]
    #[sqlx(rename="5m")]
    Five
}