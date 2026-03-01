use crate::{models::line::Line};

pub async fn get_spread_history(pool: &sqlx::PgPool, symbol: &str, exchange_pair: &str) -> Result<Vec<Line>, sqlx::Error> {    
    let lines: Vec<Line> = sqlx::query_as::<_, Line>(
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

    Ok(lines.into_iter().rev().collect())
}

pub async fn get_last_timestamp(
    pool: &sqlx::PgPool,
    ticker: String,
    exchange_pair: &str,
) -> Result<Line, sqlx::Error> {

    let line = sqlx::query_as::<_, Line>(
        r#"
        SELECT timestamp, exchange_pair, symbol, timeframe, value 
        FROM storage.lines WHERE symbol=$1 AND exchange_pair=$2
        ORDER BY timestamp DESC LIMIT 1 
        "#
    )
    .bind(ticker)
    .bind(exchange_pair)
    .fetch_one(pool)
    .await?;

    Ok(line)
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