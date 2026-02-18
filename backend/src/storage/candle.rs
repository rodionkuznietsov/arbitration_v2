use crate::models::candle::Candle;

pub async fn get_user_candles(pool: &sqlx::PgPool, symbol: &str, exchange_pair: &str) -> Result<Vec<Candle>, sqlx::Error> {
    let candles: Vec<Candle> = sqlx::query_as::<_, Candle>(
        r#"
        SELECT timestamp, exchange_pair, symbol, timeframe, open, high, low, close
        FROM storage.candles
        WHERE symbol = $1 AND exchange_pair = $2
        ORDER BY timestamp DESC
        LIMIT 100
        "#
    )
    .bind(symbol)
    .bind(exchange_pair)
    .fetch_all(pool)
    .await?;

    Ok(candles.into_iter().rev().collect())
}