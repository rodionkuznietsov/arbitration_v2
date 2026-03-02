use std::{collections::HashMap, num::NonZeroUsize, str::FromStr, time::Duration};
use chrono::{TimeZone, Timelike, Utc, Duration as ChronoDuration};
use lru::LruCache;
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::{sync::{broadcast, mpsc, oneshot}, time::{Instant, interval_at}};
use tracing::{error, info};
use crate::{models::{exchange::{ExchangeType, Spread}, line::{Line, TimeFrame}, orderbook::SnapshotUi}, storage::line_storage::{add_new_line, get_last_timestamp, get_spread_history}};

pub enum AggregatorCommand {
    UpdateQuotes {
        exchange_type: ExchangeType,
        ticker: String,
        ask: f64,
        bid: f64,
    },
    UpdateOrderbooks {
        exchange_type: ExchangeType,
        snapshot_ui: SnapshotUi,
        ticker: String
    },
    UpdateVolumes {
        exchange_type: ExchangeType,
        volume: f64,
        ticker: String
    },
    GetLinesHistory {
        exchange_pair: String,
        ticker: String,
        reply: oneshot::Sender<Vec<Line>>
    },
    GetLastTimestamp {
        exchange_pair: String,
        ticker: String,
        reply: oneshot::Sender<i64>
    },
}

#[derive(Clone, Copy, Debug)]
pub struct BestBidAsk {
    pub bid: f64,
    pub ask: f64,
}

pub struct Aggregator {
    quotes: LruCache<(ExchangeType, String), BestBidAsk>,
    pending_lines: LruCache<(String, String), Spread>,
    lines_cache: HashMap<(String, String), Vec<Line>>,
    exchanges: Vec<ExchangeType>,
    books: HashMap<(ExchangeType, String), SnapshotUi>,
    volumes: HashMap<(ExchangeType, String), f64>,

    rx: mpsc::Receiver<AggregatorCommand>,
    client_spread_tx: broadcast::Sender<(String, String, f64)>,
    pool: sqlx::PgPool,

    pub lines_cache_tx: broadcast::Sender<HashMap<(String, String), Vec<Line>>>,
    books_tx: broadcast::Sender<HashMap<(ExchangeType, String), SnapshotUi>>,
    volumes_tx: broadcast::Sender<(String, String, f64)>,
}

impl Aggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<AggregatorCommand>, 
        client_spread_tx: broadcast::Sender<(String, String, f64)>,
        pool: sqlx::PgPool,
        lines_cache_tx: broadcast::Sender<HashMap<(String, String), Vec<Line>>>,
        books_tx: broadcast::Sender<HashMap<(ExchangeType, String), SnapshotUi>>,
        volumes_tx: broadcast::Sender<(String, String, f64)>,
    ) -> Self {
        let exchanges = vec![
            ExchangeType::Gate,
            ExchangeType::Bybit
        ];

        let books = HashMap::new();
        let volumes = HashMap::new();
        
        Self { 
            quotes: LruCache::new(NonZeroUsize::new(5000).unwrap()), 
            pending_lines: LruCache::new(NonZeroUsize::new(5000).unwrap()),
            lines_cache: HashMap::new(),
            exchanges, books, volumes,

            rx: aggregator_rx, 
            client_spread_tx, 
            books_tx, volumes_tx,

            lines_cache_tx,

            pool, 
        }
    }

    pub async fn run(mut self) {
        let now = Utc::now();
        let next_min = now
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap()
            + ChronoDuration::minutes(1);

        let delay = next_min - Utc::now();
        let delay_std = Duration::from_secs(delay.num_seconds() as u64);
        let mut interval = interval_at(Instant::now() + delay_std, Duration::from_secs(60));

        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

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
                        },
                        AggregatorCommand::UpdateOrderbooks {
                            exchange_type,
                            snapshot_ui,
                            ticker
                        } => {
                            self.handle_orderbooks(
                                ticker,
                                exchange_type,
                                snapshot_ui
                            ).await;
                        },
                        AggregatorCommand::UpdateVolumes {
                            exchange_type,
                            volume,
                            ticker
                        } => {
                            self.volumes.insert((
                                exchange_type, 
                                ticker.clone()), 
                                volume
                            );
                            self.handle_volumes(ticker).await;
                        }
                        AggregatorCommand::GetLinesHistory {
                            exchange_pair,
                            ticker,
                            reply
                        } => {
                            self.get_lines_cache(
                                exchange_pair,
                                &ticker,
                                reply
                            ).await;
                        },
                        AggregatorCommand::GetLastTimestamp {
                            exchange_pair,
                            ticker,
                            reply
                         } => {
                            self.last_timestamp(
                                exchange_pair,
                                ticker,
                                reply
                            ).await
                        }
                    }
                }

                _ = interval.tick() => {
                    self.db_writer().await;
                    self.broadcast_update_lines_cache().await;
                }
            };
        }
    }

    async fn last_timestamp(
        &self,
        exchange_pair: String,
        ticker: String,
        reply: oneshot::Sender<i64>
    ) {
        let pool = self.pool.clone();
        let last_line = get_last_timestamp(&pool, ticker, &exchange_pair).await;
        if let Ok(line) = last_line {
            let timestamp = line.timestamp.timestamp();
            reply.send(timestamp).ok();
        }
    }

    async fn broadcast_update_lines_cache(&mut self) {
        if self.lines_cache_tx.send(self.lines_cache.clone()).is_err() {
            println!("Ошибка отправки lines_cache");
            return;
        }
        self.lines_cache.clear();
    }

    async fn get_lines_cache(
        &mut self,
        exchange_pair: String,
        ticker: &str,
        reply: oneshot::Sender<Vec<Line>>
    ) {
        let pool = self.pool.clone();
        let lines = get_spread_history(&pool, ticker, &exchange_pair).await;
        if let Ok(lines) = lines {
            if reply.send(lines.to_vec()).is_err() {}
        }
    }

    async fn handle_orderbooks(
        &mut self,
        ticker: String,
        exchange_type: ExchangeType,
        snapshot_ui: SnapshotUi
    ) {
        self.books.insert((
            exchange_type, ticker
        ), snapshot_ui);
        self.books_tx.send(self.books.clone()).ok();
    }

    async fn handle_volumes(
        &self,
        ticker: String
    ) {
        for i in 0..self.exchanges.len() {
            for j in (i+1)..self.exchanges.len() {
                let long_ex = self.exchanges[i];
                let short_ex = self.exchanges[j];
                
                let long_volume = self.volumes.get(&(long_ex, ticker.clone())).cloned();
                let short_volume = self.volumes.get(&(short_ex, ticker.clone())).cloned();

                if let (Some(l_vol), Some(s_vol)) = (long_volume, short_volume) {
                    let long_pair = format!("{}/{}", long_ex, short_ex);
                    // println!("{} -> {}: {}", long_pair, ticker, l_vol);

                    let short_pair = format!("{}/{}", short_ex, long_ex);
                    // println!("{} -> {}: {}", short_pair, ticker, s_vol);

                    if self.volumes_tx.send((long_pair, ticker.clone(), l_vol)).is_err() {
                        continue;
                    }

                    if self.volumes_tx.send((short_pair, ticker.clone(), s_vol)).is_err() {
                        continue;
                    }
                }
            }
        }
    }

    fn calculate_spread(&mut self, ticker: String) {
        for i in 0..self.exchanges.len() {
            for j in (i+1)..self.exchanges.len() {
                let long_ex = self.exchanges[i];
                let short_ex = self.exchanges[j];

                let long_spread = self.quotes.get(&(long_ex, ticker.clone())).cloned();
                let short_spread = self.quotes.get(&(short_ex, ticker.clone())).cloned();
                
                if let (Some(long_spread), Some(short_spread)) = (long_spread, short_spread) {
                    let mid_price  = (short_spread.bid + short_spread.ask) / 2.0;
                    let spread_in_percent = (short_spread.bid - long_spread.ask) / mid_price * 100.0;

                    let long_pair = format!("{}/{}", long_ex, short_ex);
                    self.pending_lines.put(
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
                    self.pending_lines.put(
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
    async fn db_writer(&mut self) {
        let pool = self.pool.clone();
        let all_spreads = self.pending_lines.clone();
        for ((pair, ticker), spread) in all_spreads.clone() {
            let now = Utc::now();
            let ts = now.timestamp();
            let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());
            
            let value = match BigDecimal::from_str(&format!("{}", spread.spread)) {
                Ok(p) => p,
                Err(_) => continue
            };

            let new_line = Line {
                timestamp: Utc.timestamp_opt(start_minute, 0).unwrap(),
                exchange_pair: pair.clone(),
                symbol: ticker.clone(),
                timeframe: TimeFrame::One,
                value: value
            };
            
            match add_new_line(&pool, new_line.clone()).await {
                Ok(_) => {
                    self.lines_cache.insert((pair.clone(), ticker.clone()), vec![new_line]);
                    info!("Добавлена новая линия: {} -> {}", pair, ticker);
                },
                Err(e) => {
                    error!("{}", e)
                }
            }
        }
    }
}