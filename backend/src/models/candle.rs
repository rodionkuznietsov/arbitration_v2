use sqlx::types::BigDecimal;

#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exchange_pair: String,
    pub symbol: String,
    pub interval: String,
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
}
