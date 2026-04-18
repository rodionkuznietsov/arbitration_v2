use std::{collections::HashMap, sync::Arc, time::{Duration}};
use chrono::{Timelike, Utc, Duration as ChronoDuration};
use tokio::{sync::{mpsc}, time::{Instant as TokioInstant, interval_at}};
use crate::{models::{aggregator::{Quote, SpreadPair, Volume}, exchange::ExchangeType, exchange_aggregator::{BookData, BookDataWithArc}, line::{Line, TimeFrame}, orderbook::Snapshot, websocket::Symbol}, services::{cache_aggregator::CacheAggregatorCmd, data_mapping::DataMappingCmd, exchange::exchange_aggregator::PRICE_TICK}, storage::line_storage::add_new_lines};

pub enum DataAggregatorCmd {
    MarketRegister {
        symbol: Arc<Symbol>,
        exchange_id: ExchangeType
    },
    UpdateData {
        exchange_id: ExchangeType,
        symbol: Arc<Symbol>,
        data: Arc<BookData>
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeBookData {
    pub data: Option<Arc<BookDataWithArc>>
}

/// <b>DataAggregator</b> Обьединяет данные с разных бирж
pub struct DataAggregator {
    pending_lines: HashMap<(ExchangeType, Arc<Symbol>), Arc<SpreadPair>>,
    markets: HashMap<Arc<Symbol>, HashMap<ExchangeType, ExchangeBookData>>,

    rx: mpsc::Receiver<DataAggregatorCmd>,
    data_mapping_tx: mpsc::Sender<DataMappingCmd>,
    cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,

    pool: sqlx::PgPool,
}

impl DataAggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<DataAggregatorCmd>,
        data_mapping_tx: mpsc::Sender<DataMappingCmd>,
        cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,

        pool: sqlx::PgPool,
    ) -> Self {
        let markets = HashMap::new();
        Self {
            pending_lines: HashMap::new(),
            markets,

            rx: aggregator_rx,
            data_mapping_tx,
            cache_aggregator_tx,

            pool,
        }
    }
    
    pub async fn run(
        mut self, 
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
        let mut interval = interval_at(TokioInstant::now() + delay_std, Duration::from_secs(60));

        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                Some(cmd) = self.rx.recv() => {
                    self.handle_command(cmd).await;
                }

                _ = interval.tick() => {
                    self.db_writer().await;
                },
            };
        }
    }

    async fn handle_command(
        &mut self, cmd: DataAggregatorCmd
    ) {
        match cmd {
            DataAggregatorCmd::MarketRegister { 
                symbol, 
                exchange_id 
            } => {
                // Приводим symbol к общему формату symbolusdt
                let symbol: String = symbol.chars()
                    .filter(|c| *c != '_' && *c != '-')
                    .map(|c| c.to_ascii_lowercase())
                    .collect();

                let entry = self.markets
                    .entry(Arc::new(symbol))
                    .or_insert_with(HashMap::new);

                entry.insert(exchange_id, ExchangeBookData { 
                    data: None
                });
            },
            DataAggregatorCmd::UpdateData { 
                exchange_id,
                symbol,
                mut data
            } => {
                if let Some(exchanges) = self.markets.get_mut(&symbol) {
                    if let Some(old_data) = exchanges.get_mut(&exchange_id) {
                        let data_mut = Arc::make_mut(&mut data);
                        let snapshot_arc = data_mut.snapshot.take().map(Arc::new);
                        
                        let new_data = Arc::new(
                            BookDataWithArc {
                                snapshot: snapshot_arc,
                                last_price: data.last_price,
                                volume24h: data.volume24h
                            }
                        );
                        old_data.data = Some(new_data);
                    }

                    // Обрабатываем и отправляем снапшот
                    let snapshot_data: Vec<(ExchangeType, Arc<Symbol>, (Option<Arc<Snapshot>>, Option<f64>))> = exchanges
                        .iter_mut()
                        .filter_map(|(ex_id, data)| {
                            data.data.as_ref().map(|arc| {
                                let taken_snapshot = arc.snapshot.clone();
                                (*ex_id, symbol.clone(), (taken_snapshot, arc.last_price))
                            })
                        })
                        .collect();

                    self.data_mapping_tx.send_timeout(
                        DataMappingCmd::ExchangesDataToJsonPair(
                            snapshot_data
                        ), 
                        Duration::from_millis(100)
                    ).await.ok();

                    let volumes: Vec<Volume> = exchanges
                        .iter()
                        .filter_map(|(ex_id, data)| {
                            data.data.as_ref().map(|arc| {
                                Volume { 
                                    exchange_id: *ex_id, 
                                    value: arc.volume24h,
                                    symbol: symbol.clone()
                                }
                            })
                        })
                        .collect();

                    self.data_mapping_tx.send_timeout(
                        DataMappingCmd::VolumesToJson(
                            volumes
                        ), 
                        Duration::from_millis(10)
                    ).await.ok();

                    let quotes: Vec<Option<Quote>> = exchanges
                        .iter()
                        .filter_map(|(ex_id, data)| {
                            data.data.as_ref().map(|arc| {
                                let snapshot = &arc.snapshot;
                                if let Some(snapshot) = snapshot {
                                    let best_ask = snapshot.a.iter()
                                        .min_by_key(|(price, _)| *price)
                                        .map(|(price, _)| *price as f64 / PRICE_TICK);

                                    let best_bid = snapshot.b.iter()
                                        .max_by_key(|(price, _)| *price)
                                        .map(|(price, _)| *price as f64 / PRICE_TICK);

                                    let quote = Quote {
                                        exchange_id: Some(*ex_id),
                                        symbol: Some(symbol.clone()),
                                        ask: best_ask,
                                        bid: best_bid,
                                    };
                                    return Some(quote);
                                } else {
                                    return Some(Quote::new());
                                }
                            })
                        }).collect();

                    self.calculate_spread(quotes).await;
                }
            }
        }
        
    }

    async fn calculate_spread(
        &mut self,
        quotes: Vec<Option<Quote>>,
    ) {
        let data_mapping = self.data_mapping_tx.clone();

        for (i, long_quote) in quotes.iter().enumerate() {
            for short_quote in quotes.iter().skip(i+1) {
                if let (Some(lq), Some(sq)) = (long_quote, short_quote) {
                    if let (
                        Some(symbol),
                        Some(long_exchange),
                        Some(long_ask),
                        Some(long_bid),

                        Some(short_exchange),
                        Some(short_ask), 
                        Some(short_bid),
                    ) = (
                        lq.symbol.clone(),
                        lq.exchange_id,
                        lq.ask, 
                        lq.bid,

                        sq.exchange_id,
                        sq.ask,
                        sq.bid
                    ) {
                        // Long - Short
                        let long_spread = Arc::new(self.spread_type(
                            long_exchange,
                            short_bid,

                            short_exchange,
                            long_ask,
                            symbol.clone()
                        ));

                        if let Some(err) = data_mapping.send_timeout(
                            DataMappingCmd::SpreadPairToJsonPair(long_spread.clone()), 
                            Duration::from_millis(10)
                        ).await.err() {
                            tracing::error!("DataAggregator(CalculateSpread-LongType.{}) - {err}", long_exchange, short_exchange)
                        }

                        // // Short - Long
                        // let short_spread = Arc::new(self.spread_type(
                        //     short_exchange,
                        //     long_bid,

                        //     long_exchange,
                        //     short_ask,
                        //     symbol.clone()
                        // ));

                        // if let Some(err) = data_mapping.send(
                        //     DataMappingCmd::SpreadPairToJsonPair(short_spread.clone()), 
                        // ).await.err() {
                        //     tracing::error!("DataAggregator(CalculateSpread-ShortType) - {err}")
                        // }

                        self.pending_lines.insert((long_exchange, symbol.clone()), long_spread);
                        // self.pending_lines.insert((short_exchange, symbol.clone()), short_spread);
                    }
                }
            }
        }
    }

    /// Формотирует спред к нужному направлению
    fn spread_type(
        &self,
        
        long_exchange: ExchangeType,
        long_value: f64,
        
        short_exchange: ExchangeType,
        short_value: f64,
        symbol: Arc<Symbol>,
    ) -> SpreadPair {
        let mid_in_price = (long_value + short_value) / 2.0;
        let spread_in_percent = (long_value - short_value) / mid_in_price * 100.0;

        let mid_out_price = (short_value + long_value) / 2.0;
        let spread_out_percent = (short_value - long_value) / mid_out_price * 100.0;

        let now = Utc::now();
        let timestamp = now.timestamp() - (now.timestamp() % 60);

        let spread = SpreadPair::new(
            symbol.clone(),
            long_exchange, 
            spread_in_percent, 
            short_exchange, 
            spread_out_percent, 
            timestamp
        );

        spread
    }
    
    /// Обрабатывает pending_lines и сохраняет в базу данных
    async fn db_writer(
        &mut self,
    ) {
        let pool = self.pool.clone();
        
        let mut lines = Vec::new();

        for ((_, symbol), spread) in self.pending_lines.iter() {
            let long_line = Line::new(
                spread.long_exchange, 
                spread.short_exchange, 
                spread.symbol.to_string(), 
                spread.long_spread, 
                TimeFrame::One, 
                spread.timestamp
            );

            lines.push((long_line, (spread.long_exchange, spread.short_exchange, symbol.clone())));

            let short_line = Line::new(
                spread.short_exchange, 
                spread.long_exchange, 
                spread.symbol.to_string(), 
                spread.short_spread, 
                TimeFrame::One, 
                spread.timestamp
            );

            lines.push((short_line, (spread.short_exchange, spread.long_exchange, symbol.clone())));
        }

        if !lines.is_empty() {
            match add_new_lines(&pool, &lines).await {
                Ok(_) => {
                    let cache_aggregator_tx = self.cache_aggregator_tx.clone();
                    if let Some(err) = cache_aggregator_tx.send_timeout(
                        Arc::new(CacheAggregatorCmd::AddLines { lines }), 
                        Duration::from_millis(100)
                    ).await.err() {
                        tracing::error!("DataAggregator(DbWriter) -> {err}")
                    } 
                    
                    self.pending_lines.clear();
                    tracing::info!("Данные отправлены");
                },
                Err(e) => {
                    tracing::error!("Ошибка отправки батча: {e}");
                    self.pending_lines.clear();
                }
            };
        }
    }
}