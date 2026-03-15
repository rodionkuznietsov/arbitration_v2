use std::collections::VecDeque;

use crate::models::{aggregator::KeyPair, line::Line};

pub async fn get_spread_history(pool: &sqlx::PgPool, symbol: &str, exchange_pair: &str) -> Result<VecDeque<Line>, sqlx::Error> {    
    let lines: VecDeque<Line> = sqlx::query_as::<_, Line>(
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
    .await?.into();

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

pub async fn add_new_lines(
    pool: &sqlx::PgPool, 
    lines: &Vec<(Line, KeyPair)>,
    query: &str
) -> Result<(), sqlx::Error> {
    let mut q = sqlx::query(
        query
    );

    for (v, _) in lines {
        q = q
            .bind(v.exchange_pair.clone())
            .bind(v.symbol.clone())
            .bind(v.timeframe.clone())
            .bind(v.value.clone());
    }

    q.execute(pool).await?;

    Ok(())
}