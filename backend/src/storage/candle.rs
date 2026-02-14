use crate::models::candle::Candle;

pub async fn get_user_candles(pool: &sqlx::PgPool, symbol: &str, exchange_pair: &str) -> Result<Vec<Candle>, sqlx::Error> {
    let candles = sqlx::query_as!(
        Candle,
        r#"
        SELECT timestamp, exchange_pair, symbol, interval, open, high, low, close
        FROM storage.candles
        WHERE symbol = $1 AND exchange_pair = $2
        ORDER BY timestamp DESC
        LIMIT 100
        "#,
        symbol,
        exchange_pair
    )
    .fetch_all(pool)
    .await?;

    Ok(candles.into_iter().rev().collect())
}