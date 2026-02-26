pub enum AggregatorCommand {
    UpdateQuotes {
        exchange_type: ExchangeType,
        ticker: String,
        ask: f64,
        bid: f64,
    },
}

use std::{num::NonZeroUsize, str::FromStr, time::Duration};

use chrono::Utc;
use lru::LruCache;
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};

use crate::{models::{exchange::{ExchangeType, Spread}, line::{Line, TimeFrame}}, storage::line_storage::add_new_line};

#[derive(Clone, Copy, Debug)]
pub struct BestBidAsk {
    pub bid: f64,
    pub ask: f64,
}

pub struct Aggregator {
    quotes: LruCache<(ExchangeType, String), BestBidAsk>,
    lines: LruCache<(String, String), Spread>,

    rx: mpsc::Receiver<AggregatorCommand>,
    client_spread_tx: broadcast::Sender<(String, String, f64)>,
    pool: sqlx::PgPool
}

impl Aggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<AggregatorCommand>, 
        client_spread_tx: broadcast::Sender<(String, String, f64)>,
        pool: sqlx::PgPool
    ) -> Self {
        Self { 
            quotes: LruCache::new(NonZeroUsize::new(5000).unwrap()), 
            lines: LruCache::new(NonZeroUsize::new(5000).unwrap()),
            
            rx: aggregator_rx, 
            client_spread_tx, 
            pool 
        }
    }

    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(TimeFrame::One.to_secs_u64()));
        
        loop {
            tokio::select! {
                Some(cmd) = self.rx.recv() => {
                    match cmd {
                        AggregatorCommand::UpdateQuotes { 
                            exchange_type ,
                            ticker,
                            ask,
                            bid
                        } => {
                            self.quotes.put(
                                (
                                    exchange_type, 
                                    ticker.clone()
                                ), 
                                BestBidAsk { 
                                    bid, 
                                    ask 
                                }
                            );
                            self.calculate_spread(ticker);
                        }
                    }
                }

                _ = interval.tick() => {
                    self.db_writer().await
                }
            };
        }
    }

    fn calculate_spread(&mut self, ticker: String) {
        let exchanges = vec![
            ExchangeType::Gate,
            ExchangeType::Bybit
        ];

        for i in 0..exchanges.len() {
            for j in (i+1)..exchanges.len() {
                let long_ex = exchanges[i];
                let short_ex = exchanges[j];

                let long_spread = self.quotes.get(&(long_ex, ticker.clone())).cloned();
                let short_spread = self.quotes.get(&(short_ex, ticker.clone())).cloned();
                
                if let (Some(long_spread), Some(short_spread)) = (long_spread, short_spread) {
                    let mid_price  = (short_spread.bid + short_spread.ask) / 2.0;
                    let spread_in_percent = (short_spread.bid - long_spread.ask) / mid_price * 100.0;

                    let long_pair = format!("{}/{}", long_ex, short_ex);
                    self.lines.put(
                        (long_pair.clone(), ticker.clone()), 
                        Spread { 
                            ticker: ticker.clone(), 
                            pair: long_pair.clone(), 
                            spread: OrderedFloat(spread_in_percent) 
                        }
                    );

                    // println!("Out: {} -> {}: {:.2}", format!("{}/{}", ExchangeType::Bybit, ExchangeType::Gate), ticker, spread_in_percent);

                    let mid_out_price = (long_spread.bid + short_spread.ask) / 2.0;
                    let spread_out_percent = (long_spread.bid - short_spread.ask) / mid_out_price * 100.0;

                    // println!("In: {} -> {}: {:.2}", format!("{}/{}", ExchangeType::Gate, ExchangeType::Bybit), ticker, spread_out_percent);

                    let short_pair = format!("{}/{}", short_ex, long_ex);
                    self.lines.put(
                        (short_pair.clone(), ticker.clone()), 
                        Spread { 
                            ticker: ticker.clone(), 
                            pair: short_pair.clone(), 
                            spread: OrderedFloat(spread_out_percent) 
                        }
                    );
                    
                    if self.client_spread_tx.send((long_pair.clone(), ticker.clone(), spread_in_percent)).is_err() {
                        continue;
                    }

                    if self.client_spread_tx.send((short_pair.clone(), ticker.clone(), spread_out_percent)).is_err() {
                        continue;
                    }
                }
            }
        }
    }

    /// Берет данные из <b>lines</b> и сохраняет данные в базуданных
    async fn db_writer(&self) {
        let pool = self.pool.clone();
        let all_spreads = self.lines.clone();
        for ((pair, ticker), spread) in all_spreads.clone() {
            let now = Utc::now();
            
            let value = match BigDecimal::from_str(&format!("{}", spread.spread)) {
                Ok(p) => p,
                Err(_) => continue
            };
            
            match add_new_line(&pool, Line {
                timestamp: now,
                exchange_pair: pair.clone(),
                symbol: ticker.clone(),
                timeframe: TimeFrame::One,
                value: value
            }).await {
                Ok(_) => {
                    info!("Добавлена новая линия: {} -> {}", pair, ticker)
                },
                Err(e) => {
                    error!("{}", e)
                }
            }
        }
    }
}