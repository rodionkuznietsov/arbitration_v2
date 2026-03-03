use std::{collections::HashMap, num::NonZeroUsize, str::FromStr, sync::Arc, time::Duration};
use chrono::{TimeZone, Timelike, Utc, Duration as ChronoDuration};
use lru::LruCache;
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::{sync::{broadcast, mpsc, oneshot}, time::{Instant, interval_at}};
use tracing::{error, info};
use crate::{models::{aggregator::{AggregatorEvent, AggregatorMessage}, exchange::{ExchangeType, Spread}, line::{Line, TimeFrame}, orderbook::SnapshotUi, websocket::ChartEvent}, storage::line_storage::{add_new_line, get_last_timestamp, get_spread_history}};

pub enum AggregatorCommand {
    Subscribe {
        event: AggregatorEvent,
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        ticker: String,
        response_tx: oneshot::Sender<broadcast::Receiver<AggregatorMessage>>
    },
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
}

#[derive(Clone, Copy, Debug)]
pub struct BestBidAsk {
    pub bid: f64,
    pub ask: f64,
}

pub struct Aggregator {
    subscriptions: HashMap<(AggregatorEvent, ExchangeType, ExchangeType, String), broadcast::Sender<AggregatorMessage>>,
    quotes: LruCache<(ExchangeType, String), BestBidAsk>,
    pending_lines: LruCache<(String, String), Spread>,
    lines_cache: HashMap<(String, String), Vec<Line>>,
    exchanges: Vec<ExchangeType>,
    books: HashMap<(ExchangeType, String), Arc<SnapshotUi>>,
    volumes: HashMap<(ExchangeType, String), f64>,

    rx: mpsc::Receiver<AggregatorCommand>,
    pool: sqlx::PgPool,

    pub lines_cache_tx: broadcast::Sender<HashMap<(String, String), Vec<Line>>>,
}

impl Aggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<AggregatorCommand>, 
        pool: sqlx::PgPool,
        lines_cache_tx: broadcast::Sender<HashMap<(String, String), Vec<Line>>>,
    ) -> Self {
        let exchanges = vec![
            ExchangeType::Gate,
            ExchangeType::Bybit
        ];

        let books = HashMap::new();
        let volumes = HashMap::new();
        
        Self {
            subscriptions: HashMap::new(),
            quotes: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            pending_lines: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            lines_cache: HashMap::new(),
            exchanges,
            books,
            volumes,

            rx: aggregator_rx,
            pool,

            lines_cache_tx,
        }
    }
    
    pub fn subscribe(
        &mut self,
        event: AggregatorEvent,
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        ticker: String,
        response_tx: oneshot::Sender<broadcast::Receiver<AggregatorMessage>>
    ) {
        let tx = self.subscriptions.entry((event, long_exchange, short_exchange, ticker)).or_insert_with(|| {
            let (tx, _) = broadcast::channel::<AggregatorMessage>(1000);
            tx
        });

        let subscribe = tx.subscribe();
        let _ = response_tx.send(subscribe);
    }

    pub async fn run(mut self) {
        // Поток для отправки orderbooks клиентам, которые подписались на них, 
        // но с конкретным токеном, а не всеми подряд
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
                    self.handle_command(cmd).await;
                }

                _ = interval.tick() => {
                    self.db_writer().await;
                    self.broadcast_update_lines_cache().await;
                }
            };

            // Отправляем данные клиентам, которые подписались на них
            self.stream_data_to_subscribers().await;
        }
    }

    async fn handle_command(&mut self, cmd: AggregatorCommand) {
        match cmd {
            AggregatorCommand::Subscribe {
                event,
                long_exchange,
                short_exchange,
                ticker,
                response_tx
            } => {
                self.subscribe(
                    event,
                    long_exchange, 
                    short_exchange, 
                    ticker,
                    response_tx
                );
            },
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
                self.books.insert((
                    exchange_type, ticker.clone()
                ), Arc::new(snapshot_ui));
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
            },
        }
    }

    async fn stream_data_to_subscribers(
        &mut self,
    ) {
        for ((
                event, 
                long_exchange, 
                short_exchange, 
                ticker
            ), 
            tx
        ) in self.subscriptions.iter() {            
            match event {
                AggregatorEvent::OrderBook => {
                    println!("{}", tx.receiver_count());

                    let long_book = self.books.get(&(*long_exchange, ticker.clone()));
                    let short_book = self.books.get(&(*short_exchange, ticker.clone()));

                    if let (Some(long_book), Some(short_book)) = (long_book, short_book) {
                        let msg = AggregatorMessage { 
                            long_value: None, 
                            short_value: None,
                            long_order_book: Some(Arc::clone(long_book)),
                            short_order_book: Some(Arc::clone(short_book))
                        };
                        tx.send(msg).ok();
                    }
                },
                AggregatorEvent::ChartEvent(chart_event) => {
                    match chart_event {
                        ChartEvent::Volume24hr => {
                            let long_volume = self.volumes.get(&(*long_exchange, ticker.clone()));
                            let short_volume = self.volumes.get(&(*short_exchange, ticker.clone()));

                            if let (Some(long_volume), Some(short_volume)) = (long_volume, short_volume) {
                                let msg = AggregatorMessage { 
                                    long_value: Some(*long_volume), 
                                    short_value: Some(*short_volume),
                                    long_order_book: None,
                                    short_order_book: None
                                };
                                tx.send(msg).ok();
                            }
                        },
                        _ => {}
                    }
                }
            }
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
        self.lines_cache_tx.send(self.lines_cache.clone()).ok();
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

    async fn get_volumes(
        &self,
        long_exchange: ExchangeType,
        short_exchange: ExchangeType,
        ticker: String
    ) {
        let long_volume = self.volumes.get(&(long_exchange, ticker.clone()));
        let short_volume = self.volumes.get(&(short_exchange, ticker.clone()));
        
        if let (Some(l_vol), Some(s_vol)) = (long_volume, short_volume) {
            let key = (AggregatorEvent::ChartEvent(ChartEvent::Volume24hr), long_exchange, short_exchange, ticker.clone());
            if let Some(tx) = self.subscriptions.get(&key) {
                tx.send(AggregatorMessage { 
                    long_value: Some(*l_vol), 
                    short_value: Some(*s_vol),
                    long_order_book: None,
                    short_order_book: None,
                }).ok();
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

                    let mid_out_price = (long_spread.bid + short_spread.ask) / 2.0;
                    let spread_out_percent = (long_spread.bid - short_spread.ask) / mid_out_price * 100.0;

                    let short_pair = format!("{}/{}", short_ex, long_ex);
                    self.pending_lines.put(
                        (short_pair.clone(), ticker.clone()), 
                        Spread { 
                            ticker: ticker.clone(), 
                            pair: short_pair.clone(), 
                            spread: OrderedFloat(spread_out_percent) 
                        }
                    );
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