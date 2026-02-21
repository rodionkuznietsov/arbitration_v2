use crate::models::line::Line;

pub async fn get_spread_history(pool: &sqlx::PgPool, symbol: &str, exchange_pair: &str) -> Result<Vec<Line>, sqlx::Error> {
    let candles: Vec<Line> = sqlx::query_as::<_, Line>(
        r#"
        SELECT timestamp, exchange_pair, symbol, timeframe, value
        FROM storage.lines
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

/// Возращает последний спред каждой пары
pub async fn get_last_spread_of_all_exhchange_pairs(
    pool: &sqlx::PgPool
) -> Result<Vec<Line>, sqlx::Error> {
    let lines: Vec<Line> = sqlx::query_as::<_, Line>(
        r#"
        SELECT DISTINCT ON (exchange_pair) *
        FROM storage.lines
        ORDER BY exchange_pair, timestamp DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(lines)
}

pub async fn add_new_line(pool: &sqlx::PgPool, line: Line) -> Result<(), sqlx::Error> {
    let symbol = line.symbol;
    let exchange_pair = line.exchange_pair;
    let timestamp = line.timestamp;
    let value = line.value;
    let timeframe = line.timeframe;

    let _ = sqlx::query(
        r#"
            INSERT INTO storage.lines
                (timestamp, exchange_pair, symbol, timeframe, value)
            VALUES 
                ($1, $2, $3, $4, $5)
        "#
    )
    .bind(timestamp)
    .bind(exchange_pair)
    .bind(symbol)
    .bind(timeframe)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}