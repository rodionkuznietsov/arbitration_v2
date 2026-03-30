use std::{collections::{HashMap, VecDeque}, sync::Arc};

use crate::models::{exchange::ExchangeType, line::Line, websocket::Symbol};

pub async fn get_spread_history(
    pool: &sqlx::PgPool, 
    symbol: &str, 
    long_exchange: ExchangeType,
    short_exchange: ExchangeType,
) -> Result<HashMap<(ExchangeType, ExchangeType, Arc<Symbol>), VecDeque<Line>>, sqlx::Error> {    
    let lines: Vec<Line> = sqlx::query_as::<_, Line>(
        r#"
        SELECT timestamp, long_exchange, short_exchange, symbol, timeframe, value
        FROM storage.lines
        WHERE symbol = $1 
            AND (
                (long_exchange = $2 AND short_exchange = $3)
                OR
                (long_exchange = $3 AND short_exchange = $2)
            )
        ORDER BY timestamp ASC
        LIMIT 200
        "#
    )
    .bind(symbol)
    .bind(long_exchange)
    .bind(short_exchange)
    .fetch_all(pool)
    .await?;

    let mut history_map = HashMap::new();
    for line in lines.clone() {
        let key = (line.long_exchange, line.short_exchange, Arc::new(symbol.to_string()));
        history_map
            .entry(key)
            .or_insert_with(VecDeque::new)
            .push_back(line);
    }

    Ok(history_map)
}

pub async fn add_new_lines(
    pool: &sqlx::PgPool, 
    lines: &Vec<(Line, (ExchangeType, ExchangeType, Arc<Symbol>))>,
    query: &str
) -> Result<(), sqlx::Error> {
    let mut q = sqlx::query(
        query
    );

    for (line, (_, _, symbol)) in lines {
        q = q
            .bind(line.timestamp)
            .bind(line.long_exchange.clone())
            .bind(line.short_exchange.clone())
            .bind(symbol.to_string())
            .bind(line.timeframe.clone())
            .bind(line.value.clone());
    }

    q.execute(pool).await?;

    Ok(())
}