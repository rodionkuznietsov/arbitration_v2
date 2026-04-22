use std::{collections::{HashMap, VecDeque}, sync::Arc};

use sqlx::QueryBuilder;

use crate::models::{exchange::ExchangeType, line::Line, websocket::Symbol};

pub async fn get_spread_history(
    pool: &Option<sqlx::PgPool>, 
    symbol: &str, 
    long_exchange: ExchangeType,
    short_exchange: ExchangeType,
) -> Result<HashMap<(ExchangeType, ExchangeType, Arc<Symbol>), VecDeque<Line>>, sqlx::Error> {    
    if let Some(pool) = pool {
        
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

        return Ok(history_map);
    }

    Ok(HashMap::new())
}

pub async fn add_new_lines(
    pool: &Option<sqlx::PgPool>, 
    lines: &Vec<(Line, (ExchangeType, ExchangeType, Arc<Symbol>))>,
) -> Result<(), sqlx::Error> {
    if let Some(pool) = pool {
        let mut builder = QueryBuilder::new(
            "INSERT INTO storage.lines (timestamp, long_exchange, short_exchange, symbol, timeframe, value) "
        );

        for chunk in lines.chunks(1000) {
            builder.push_values(chunk.iter(), |mut b, (line, (_, _, symbol))| {
                b.push_bind(line.timestamp)
                    .push_bind(line.long_exchange)
                    .push_bind(line.short_exchange)
                    .push_bind(symbol.to_string())
                    .push_bind(line.timeframe.clone())
                    .push_bind(line.value);
            });
        }

        let q = builder.build();
        let result = q.execute(pool).await?;

        tracing::info!("Вставленные rows: {}; expected: {}", result.rows_affected(), lines.len());
    }
    
    Ok(())
}