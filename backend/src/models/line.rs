use serde::{Deserialize, Serialize};
use sqlx::{Type, prelude::FromRow, types::BigDecimal};

#[derive(Debug, Clone, FromRow)]
pub struct Line {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange_pair: String,
    pub symbol: String,
    pub timeframe: TimeFrame,
    pub value: BigDecimal
}

#[derive(Debug, Clone, Deserialize, Serialize, Type)]
#[sqlx(type_name="timeframe")]
pub enum TimeFrame {
    #[serde(rename="1m")]
    #[sqlx(rename="1m")]
    One
}

impl TimeFrame {
    pub fn to_secs_i64(&self) -> i64 {
        match self {
            Self::One => 60
        }
    }
}