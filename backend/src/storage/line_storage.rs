use std::collections::{HashMap, VecDeque};

use crate::models::{aggregator::KeyPair, exchange::ExchangeType, line::Line, websocket::Symbol};

pub async fn get_all_spread_history(pool: &sqlx::PgPool) -> Result<HashMap<(ExchangeType, ExchangeType, Symbol), VecDeque<Line>>, sqlx::Error> {    
    let mut grouped: HashMap<(ExchangeType, ExchangeType, Symbol), VecDeque<Line>> = HashMap::new();
    
    let lines: Vec<Line> = sqlx::query_as::<_, Line>(
        r#"
        SELECT timestamp, long_exchange, short_exchange, symbol, timeframe, value
        FROM storage.lines
        ORDER BY timestamp DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool)
    .await?;

    for line in lines {
        grouped
            .entry((line.long_exchange, line.short_exchange, line.symbol.clone()))
            .or_default()
            .push_back(line)
    }

    Ok(grouped)
}

pub async fn get_spread_history(
    pool: &sqlx::PgPool, 
    symbol: &str, 
    long_exchange: ExchangeType,
    short_exchange: ExchangeType,
) -> Result<VecDeque<Line>, sqlx::Error> {    
    // let lines: VecDeque<Line> = sqlx::query_as::<_, Line>(
    //     r#"
    //     SELECT timestamp, long_exchange, short_exchange, symbol, timeframe, value
    //     FROM storage.lines
    //     WHERE symbol = $1 AND exchange_pair = $2
    //     ORDER BY timestamp DESC
    //     LIMIT 100
    //     "#
    // )
    // .bind(symbol)
    // .bind(long_exchange)
    // .bind(short_exchange)
    // .fetch_all(pool)
    // .await?.into();

    Ok(VecDeque::new())
}

pub async fn add_new_lines(
    pool: &sqlx::PgPool, 
    lines: &Vec<(Line, KeyPair)>,
    query: &str
) -> Result<(), sqlx::Error> {
    // let mut q = sqlx::query(
    //     query
    // );

    // for (v, _) in lines {
    //     q = q
    //         .bind(v.timestamp)
    //         .bind(v.long_exchange.clone())
    //         .bind(v.short_exchange.clone())
    //         .bind(v.symbol.clone())
    //         .bind(v.timeframe.clone())
    //         .bind(v.value.clone());
    // }

    // q.execute(pool).await?;

    Ok(())
}