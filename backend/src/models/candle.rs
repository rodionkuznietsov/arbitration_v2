use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

#[derive(Debug, Clone)]
pub struct Candle {
    pub exchange: String,
    pub symbol: String,
    pub interval: String,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub user_id: String,
}
