use std::{str::FromStr, sync::Arc, time::Duration};
use chrono::{TimeZone, Utc};
use dashmap::DashMap;
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::sync::{mpsc};
use crate::{exchanges::{bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket}, models::{exchange::{ExchangeType, SharedSpreads, Spread}, line::{Line, TimeFrame}}, services::market_manager::ExchangeWebsocket, storage::line_storage::get_last_spread_of_all_exhchange_pairs};

pub fn spawn_local_spread_engine(
    bybit: Arc<BybitWebsocket>,
    gate: Arc<GateWebsocket>,
    kucoin: Arc<KuCoinWebsocket>,
    shared_spreads: Arc<SharedSpreads>
) {
    let (spread_tx, mut spread_rx) = mpsc::channel::<Option<(ExchangeType, String, Option<f64>, Option<f64>)>>(1);

    tokio::spawn({
        async move {
            loop {
                bybit.clone().get_spread(spread_tx.clone()).await;
                gate.clone().get_spread(spread_tx.clone()).await;
                kucoin.clone().get_spread(spread_tx.clone()).await;
            }
        }
    });

    tokio::spawn({
        async move {
            while let Some(Some((
                exchange_type, 
                ticker,
                ask, 
                bid)
            )) = spread_rx.recv().await {
                let spreads = shared_spreads.clone();

                let ask = match ask {
                    Some(a) => a,
                    None => continue
                };

                let bid = match bid {
                    Some(b) => b,
                    None => continue
                };
                
                spreads.exchange.insert(exchange_type, 
                    Spread {
                        ask: OrderedFloat(ask),
                        bid: OrderedFloat(bid),
                        ticker: ticker
                    }
                );
            }
        }
    });
}


pub fn calculate_spread_for_chart(
    shared_spreads: Arc<SharedSpreads>,
    spread_tx: Arc<tokio::sync::broadcast::Sender<(String, String, OrderedFloat<f64>)>>
) {
    tokio::spawn(async move {
        let spread_tx = spread_tx.clone();
        loop {
            let spreads = shared_spreads.clone();
            let exchanges: Vec<(ExchangeType, Spread)> = spreads.exchange.iter().map(|item| {
                (*item.key(), item.value().clone())
            }).collect();

            for i in 0..exchanges.len() {
                for j in (i+1)..exchanges.len() {
                    let (long_exchange, long_spread) = &exchanges[i];
                    let (short_exchange, short_spread) = &exchanges[j];

                    let mid_price  = (short_spread.bid + short_spread.ask) / 2.0;
                    let spread_in_percent = (short_spread.bid - long_spread.ask) / mid_price * 100.0;
                    if spread_tx.send((
                        format!("{}/{}", long_exchange, short_exchange), 
                        long_spread.ticker.clone(),
                        spread_in_percent,
                    )).is_err() {
                        continue;
                    }

                    println!("Out: {} -> {:.2}", format!("{}/{}", long_exchange, short_exchange), spread_in_percent);

                    let mid_out_price = (long_spread.bid + short_spread.ask) / 2.0;
                    let spread_out_percent = (long_spread.bid - short_spread.ask) / mid_out_price * 100.0;
                    if spread_tx.send((
                        format!("{}/{}", short_exchange, long_exchange), 
                        short_spread.ticker.clone(),
                        spread_out_percent
                    )).is_err() {
                        continue;
                    }

                    println!("In: {} -> {:.2}", format!("{}/{}", short_exchange, long_exchange), spread_out_percent);

                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });
}

/// В фоне обрабатываем спред с желаемым <b>TimeFrame</b>.<br>
/// Через каждую <b>TimeFrame</b> минуту данные последнего спреда записываются в базу данных.
pub fn spawn_spread_db_wirter(
    timeframe: TimeFrame,
    spread_tx: tokio::sync::broadcast::Sender<(String, String, OrderedFloat<f64>)>,
    new_line_tx: mpsc::Sender<Line>,
    pool: sqlx::PgPool
) {
    tokio::spawn(async move {
        let init_lines = get_last_spread_of_all_exhchange_pairs(&pool).await;
        let last_minute_map = DashMap::new();

        if let Ok(lines) = init_lines {
            for line in lines.clone() {
                let key = line.exchange_pair.clone();
                let last_time = line.timestamp;
                let last_timestamp = last_time.timestamp();
                let last_minute = last_timestamp - (last_timestamp % 60);

                last_minute_map.insert(key.clone(), last_minute);
            }
        }
        
        let mut spread_rx = spread_tx.subscribe();

        loop {
            while let Ok((pair, ticker, spread)) = spread_rx.recv().await {
                if let Some(mut last_minute) = last_minute_map.get_mut(&pair) {
                    let now = Utc::now();
                    let ts = now.timestamp();
                    let start_time = ts - (ts % timeframe.to_secs_i64());

                    if start_time > *last_minute {
                        *last_minute = start_time;

                        let new_line = Line {
                            timestamp: Utc.timestamp_opt(*last_minute, 0).unwrap(),
                            exchange_pair: pair,
                            symbol: ticker.clone(),
                            timeframe: TimeFrame::One,
                            value: BigDecimal::from_str(&format!("{}", spread)).unwrap()
                        };

                        if new_line_tx.send(new_line.clone()).await.is_err() {
                            continue
                        }
                    }
                } else {        
                    let now = Utc::now();
                    let ts = now.timestamp();
                    let start_time = ts - (ts % timeframe.to_secs_i64());

                    let new_line = Line {
                        timestamp: now,
                        exchange_pair: pair.clone(),
                        symbol: ticker.clone(),
                        timeframe: TimeFrame::One,
                        value: BigDecimal::from_str(&format!("{}", spread)).unwrap()
                    };

                    println!("Нет записей для: {}", new_line.exchange_pair);

                    if new_line_tx.send(new_line.clone()).await.is_err() {
                        continue
                    }

                    last_minute_map.insert(pair, start_time);
                }
            }
        }
    });
}