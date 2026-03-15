use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow, types::BigDecimal};

use crate::models::{websocket::Symbol};

#[derive(Debug, Clone, FromRow)]
pub struct Line {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange_pair: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub value: BigDecimal
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