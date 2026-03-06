use std::{collections::HashMap, num::NonZeroUsize, str::FromStr, sync::Arc, time::Duration};
use chrono::{TimeZone, Timelike, Utc, Duration as ChronoDuration};
use lru::LruCache;
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::{sync::{mpsc}, time::{Instant, interval_at}};
use tracing::{error, info};
use crate::{models::{aggregator::{AggregatorPayload, ClientAggregatorUse}, exchange::{ExchangeType, Spread}, line::{Line, TimeFrame}, orderbook::SnapshotUi, websocket::{ChannelSubscription, Symbol}}, storage::line_storage::add_new_line, transport::client_aggregator::ClientAggregatorCmd};

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
}

#[derive(Clone, Copy, Debug)]
pub struct BestBidAsk {
    pub bid: f64,
    pub ask: f64,
}

pub struct DataAggregator {
    quotes: LruCache<(ExchangeType, String), BestBidAsk>,
    pending_lines: LruCache<(String, String), Spread>,
    lines_cache: HashMap<(String, String), Vec<Line>>,
    exchanges: Vec<ExchangeType>,
    books: HashMap<Arc<Symbol>, HashMap<ExchangeType, Arc<SnapshotUi>>>,
    volumes: HashMap<(ExchangeType, String), f64>,

    rx: mpsc::Receiver<AggregatorCommand>,
    pool: sqlx::PgPool,
}

impl DataAggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<AggregatorCommand>, 
        pool: sqlx::PgPool,
    ) -> Self {
        let exchanges = vec![
            ExchangeType::Gate,
            ExchangeType::Bybit,
            ExchangeType::KuCoin,
        ];

        let books = HashMap::new();
        let volumes = HashMap::new();
        
        Self {
            quotes: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            pending_lines: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            lines_cache: HashMap::new(),
            exchanges,
            books,
            volumes,

            rx: aggregator_rx,
            pool,
        }
    }
    
    pub async fn run(
        mut self, 
        client_aggregator_tx: mpsc::Sender<ClientAggregatorCmd>
    ) {
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
                }
            };

            // Создаем Payload для отправки клиентам, которые подписались на обновления, и отправляем его
            self.send_payload_to_client_aggregator(client_aggregator_tx.clone()).await;
        }
    }

    async fn handle_command(&mut self, cmd: AggregatorCommand) {
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
                let entry = self.books
                    .entry(Arc::new(ticker))
                    .or_insert_with(HashMap::new);

                entry.insert(exchange_type, Arc::new(snapshot_ui));
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

    async fn send_payload_to_client_aggregator(
        &mut self,
        client_aggregator_tx: mpsc::Sender<ClientAggregatorCmd>
    ) {
        for (symbol, exchanges) in &self.books {
            for (i, (long_ex, long_book)) in exchanges.iter().enumerate() {
                for (short_ex, short_book) in exchanges.iter().skip(i+1) {
                    let long_key = ChannelSubscription::OrderBook { 
                        long_exchange: *long_ex, 
                        short_exchange: *short_ex, 
                        ticker: symbol.clone()
                    };

                    let long_payload = Arc::new(AggregatorPayload::OrderBook { 
                        long_order_book: long_book.clone(), 
                        short_order_book: short_book.clone(), 
                        ticker: symbol.clone() 
                    });

                    client_aggregator_tx.try_send(ClientAggregatorCmd::Use(
                        ClientAggregatorUse::Publish { 
                            key: long_key,
                            payload: long_payload
                        }
                    )).ok();

                    let short_key = ChannelSubscription::OrderBook { 
                        long_exchange: *short_ex, 
                        short_exchange: *long_ex, 
                        ticker: symbol.clone()
                    };

                    let short_payload = Arc::new(AggregatorPayload::OrderBook { 
                        long_order_book: short_book.clone(), 
                        short_order_book: long_book.clone(), 
                        ticker: symbol.clone() 
                    });

                    client_aggregator_tx.try_send(ClientAggregatorCmd::Use(
                        ClientAggregatorUse::Publish { 
                            key: short_key,
                            payload: short_payload
                        }
                    )).ok();
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