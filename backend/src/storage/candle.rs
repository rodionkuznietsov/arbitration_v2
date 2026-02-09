use crate::models::candle::Candle;

pub async fn get_user_candles(pool: &sqlx::PgPool, symbol: &str, user_id: &str) -> Result<Vec<Candle>, sqlx::Error> {
    let candles = sqlx::query_as!(
        Candle,
        r#"
        SELECT exchange, symbol, interval, open, high, low, close, user_id
        FROM storage.candles
        WHERE symbol = $1 AND user_id = $2
        ORDER BY timestamp ASC
        "#,
        symbol,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(candles)
}