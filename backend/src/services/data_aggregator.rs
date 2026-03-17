use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use chrono::{Timelike, Utc, Duration as ChronoDuration};
use tokio::{sync::{broadcast, mpsc, watch}, time::{Instant as TokioInstant, interval_at}};
use crate::{models::{aggregator::{AggregatorPayload, ClientAggregatorUse, KeyMarketType, KeyPair, SpreadPair}, exchange::{ExchangeType}, line::{Line, TimeFrame}, orderbook::SnapshotUi, websocket::{ChannelSubscription, ChartEvent, Symbol}}, services::cache_aggregator::CacheAggregatorCmd, storage::line_storage::add_new_lines, transport::client_aggregator::ClientAggregatorCmd};

pub enum DataAggregatorCmd {
    MarketRegister {
        symbol: Arc<Symbol>,
        exchange_id: ExchangeType
    }, 
    UpdateQuotes {
        exchange_id: ExchangeType,
        symbol: Arc<Symbol>,
        ask: f64,
        bid: f64,
    },
    UpdateOrderbooks {
        exchange_id: ExchangeType,
        snapshot_ui: SnapshotUi,
        symbol: Arc<Symbol>,
        _created_at: Instant
    },
    UpdateVolumes {
        exchange_id: ExchangeType,
        volume: f64,
        symbol: Arc<Symbol>
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Quote {
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug)]
pub struct ExchangeBookData {
    pub snapshot: Option<Arc<SnapshotUi>>,
    pub quote: Option<Quote>,
    pub volume24h: Option<f64>,
}

pub struct DataAggregator {
    pending_lines: HashMap<KeyPair, SpreadPair>,
    markets: HashMap<Arc<Symbol>, HashMap<ExchangeType, ExchangeBookData>>,

    rx: mpsc::Receiver<DataAggregatorCmd>,
    cache_aggregator_tx: mpsc::Sender<CacheAggregatorCmd>,
    markets_updated: broadcast::Sender<Arc<Symbol>>,

    pool: sqlx::PgPool,
}

// Исправить дефект с базой данных
impl DataAggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<DataAggregatorCmd>,
        cache_aggregator_tx: mpsc::Sender<CacheAggregatorCmd>,

        pool: sqlx::PgPool,
    ) -> Self {
        let markets = HashMap::new();
        let (markets_updated, _) = broadcast::channel(32);
        
        Self {
            pending_lines: HashMap::new(),
            markets,

            rx: aggregator_rx,
            cache_aggregator_tx,
            markets_updated,

            pool,
        }
    }
    
    pub async fn run(
        mut self, 
        client_aggregator_tx: watch::Sender<Arc<ClientAggregatorCmd>>
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
        
        let (spread_tx, mut spread_rx) = mpsc::channel::<(KeyPair, SpreadPair)>(1024);
        let mut markets_updated_rx = self.markets_updated.subscribe();

        loop {
            tokio::select! {
                Some(cmd) = self.rx.recv() => {
                    self.handle_command(cmd).await;
                }

                _ = interval.tick() => {
                    self.db_writer().await;
                },

                Ok(symbol) = markets_updated_rx.recv() => {
                    self.send_payload_to_client_aggregator(
                        client_aggregator_tx.clone(), 
                        spread_tx.clone(),
                        symbol.clone(),
                    ).await;
                }

                Some((key_pair, spread)) = spread_rx.recv() => {
                    self.pending_lines.insert(
                        key_pair, 
                        spread
                    );
                }
            };
        }
    }

    async fn handle_command(&mut self, cmd: DataAggregatorCmd) {
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
                    snapshot: None, 
                    quote: None, 
                    volume24h: None,
                });
            },
            DataAggregatorCmd::UpdateQuotes { 
                exchange_id,
                symbol,
                ask,
                bid
            } => {
                if let Some(exchange_map) = self.markets.get_mut(&symbol) {
                    if let Some(data) = exchange_map.get_mut(&exchange_id) {
                        data.quote = Some(Quote { bid, ask });
                        _ = self.markets_updated.send(symbol);
                    }                    
                }
            },
            DataAggregatorCmd::UpdateOrderbooks {
                exchange_id,
                snapshot_ui,
                symbol,
                _created_at
            } => {
                if let Some(exchange_map) = self.markets.get_mut(&symbol) {
                    if let Some(data) = exchange_map.get_mut(&exchange_id) {                        
                        data.snapshot = Some(Arc::new(snapshot_ui));
                        _ = self.markets_updated.send(symbol);
                    }                    
                }
            },
            DataAggregatorCmd::UpdateVolumes {
                exchange_id,
                volume,
                symbol
            } => {
                if let Some(exchange_map) = self.markets.get_mut(&symbol) {
                    if let Some(data) = exchange_map.get_mut(&exchange_id) {
                        data.volume24h = Some(volume);
                        _ = self.markets_updated.send(symbol);
                    }
                }
            },
        }
    }

    async fn send_payload_to_client_aggregator(
        &mut self,
        client_aggregator_tx: watch::Sender<Arc<ClientAggregatorCmd>>,
        spread_tx: mpsc::Sender<(KeyPair, SpreadPair)>,
        symbol: Arc<Symbol>
    ) {
        let data = self.markets.get(&symbol);
        if let Some(exchanges) = data {
            for (i, (long_ex, long_data)) in exchanges.iter().enumerate() {
                for (short_ex, short_data) in exchanges.iter().skip(i+1) {
                    let long_book = &long_data.snapshot;
                    let short_book = &short_data.snapshot;

                    if let (Some(long), Some(short)) = (long_book, short_book) {
                        let long_key = ChannelSubscription::OrderBook { 
                            long_exchange: *long_ex, 
                            short_exchange: *short_ex, 
                            ticker: symbol.clone()
                        };
                        
                        let long_payload = Arc::new(AggregatorPayload::OrderBook { 
                            long_order_book: long.clone(), 
                            short_order_book: short.clone(), 
                            ticker: symbol.clone(),
                        });

                        let short_key = ChannelSubscription::OrderBook { 
                            long_exchange: *short_ex, 
                            short_exchange: *long_ex, 
                            ticker: symbol.clone()
                        };

                        let short_payload = Arc::new(AggregatorPayload::OrderBook { 
                            long_order_book: short.clone(), 
                            short_order_book: long.clone(), 
                            ticker: symbol.clone(),
                        });
                        
                        client_aggregator_tx.send(Arc::new(ClientAggregatorCmd::Use(
                            ClientAggregatorUse::Publish { 
                                key: long_key,
                                payload: long_payload
                            }
                        ))).ok();

                        client_aggregator_tx.send(Arc::new(ClientAggregatorCmd::Use(
                            ClientAggregatorUse::Publish { 
                                key: short_key,
                                payload: short_payload
                            }
                        ))).ok();

                    }

                    let long_volume = long_data.volume24h;
                    let short_volume = short_data.volume24h;

                    if let (
                        Some(long_volume), 
                        Some(short_volume)
                    ) = (
                        long_volume, 
                        short_volume
                    ) {
                        let long_key = ChannelSubscription::Chart { 
                            long_exchange: *long_ex, 
                            short_exchange: *short_ex, 
                            ticker: symbol.clone()
                        };

                        let long_payload = Arc::new(AggregatorPayload::ChartEvent { 
                            event: ChartEvent::Volume24hr(long_volume, short_volume), 
                            ticker: symbol.clone()
                        });

                        let short_key = ChannelSubscription::Chart { 
                            long_exchange: *short_ex, 
                            short_exchange: *long_ex, 
                            ticker: symbol.clone()
                        };

                        let short_payload = Arc::new(AggregatorPayload::ChartEvent { 
                            event: ChartEvent::Volume24hr(short_volume, long_volume), 
                            ticker: symbol.clone()
                        });

                        client_aggregator_tx.send(Arc::new(ClientAggregatorCmd::Use(
                            ClientAggregatorUse::Publish { 
                                key: long_key,
                                payload: long_payload
                            }
                        ))).ok();

                        client_aggregator_tx.send(Arc::new(ClientAggregatorCmd::Use(
                            ClientAggregatorUse::Publish { 
                                key: short_key,
                                payload: short_payload
                            }
                        ))).ok();
                    }

                    if let (
                        Some(long_quote), 
                        Some(short_quote)
                    ) = (
                        long_data.quote, 
                        short_data.quote
                    ) {
                        self.calculate_spread(
                            spread_tx.clone(),
                            symbol.clone(),
                            *long_ex,
                            long_quote,
                            *short_ex,
                            short_quote,
                            client_aggregator_tx.clone()
                        ).await;
                    }
                }
            }
        }
    }

    async fn calculate_spread(
        &self,
        spread_tx: mpsc::Sender<(KeyPair, SpreadPair)>,

        symbol: Arc<Symbol>,

        long_exchange: ExchangeType,
        long_quote: Quote,

        short_exchange: ExchangeType,
        short_quote: Quote,
        client_aggregator_tx: watch::Sender<Arc<ClientAggregatorCmd>>
    ) {
        let mid_in_price = (short_quote.bid + long_quote.ask) / 2.0;
        let spread_in_percent = (short_quote.bid - long_quote.ask) / mid_in_price * 100.0;
        
        let mid_out_price = (long_quote.ask + short_quote.bid) / 2.0;
        let spread_out_percent = (long_quote.ask - short_quote.bid) / mid_out_price * 100.0;

        let now = Utc::now();
        let timestamp = now.timestamp() - (now.timestamp() % 60);
        
        spread_tx.send((
            KeyPair::new(
                KeyMarketType::new(long_exchange, short_exchange, symbol.clone()),
                KeyMarketType::new(short_exchange, long_exchange, symbol.clone()),
            ),
            SpreadPair::new(
                spread_in_percent, 
                spread_out_percent, 
                timestamp
            )
        )).await.ok();

        let long_key = ChannelSubscription::Chart { 
            long_exchange, 
            short_exchange, 
            ticker: symbol.clone()
        };

        let long_payload = Arc::new(
            AggregatorPayload::ChartEvent { 
                event: ChartEvent::UpdateLine(
                    spread_in_percent, 
                    spread_out_percent,
                    timestamp
                ),
                ticker: symbol.clone()
            }
        );
        
        client_aggregator_tx.send(Arc::new(
            ClientAggregatorCmd::Use(
                ClientAggregatorUse::Publish { 
                    key: long_key, 
                    payload: long_payload 
                }
            )
        )).ok();

        let short_key = ChannelSubscription::Chart { 
            long_exchange: short_exchange, 
            short_exchange: long_exchange, 
            ticker: symbol.clone()
        };

        let short_payload = Arc::new(
            AggregatorPayload::ChartEvent { 
                event: ChartEvent::UpdateLine(
                    spread_out_percent, 
                    spread_in_percent,
                    timestamp
                ),
                ticker: symbol
            }
        );
        
        client_aggregator_tx.send(Arc::new(
            ClientAggregatorCmd::Use(
                ClientAggregatorUse::Publish { 
                    key: short_key, 
                    payload: short_payload 
                }
            )
        )).ok();
    }

    /// Обрабатывает pending_lines и сохраняет в базу данных
    async fn db_writer(&mut self) {
        let pool = self.pool.clone();
        
        let mut placeholders: Vec<String> = Vec::new();
        let mut lines = Vec::new();

        for (i, (key_pair, spread_pair)) in self.pending_lines.iter().enumerate() {
            
            let step = i*5;            
            let long_exchange_pair = format!("{}/{}", key_pair.long_market_type.long_exchange, key_pair.long_market_type.short_exchange);
            let short_exchange_pair = format!("{}/{}", key_pair.short_market_type.long_exchange, key_pair.short_market_type.short_exchange);

            let long_line = Line::new(
                long_exchange_pair, 
                key_pair.long_market_type.symbol.to_string(), 
                spread_pair.long_spread, 
                TimeFrame::One, 
                spread_pair.timestamp * 1000
            );

            let short_line = Line::new(
                short_exchange_pair, 
                key_pair.short_market_type.symbol.to_string(), 
                spread_pair.short_spread, 
                TimeFrame::One, 
                spread_pair.timestamp * 1000
            );

            lines.push((long_line, key_pair.clone()));
            lines.push((short_line, key_pair.clone()));

            // Создаем плейсхолдеры для sql_query
            let placeholder = format!("(${}, ${}, ${}, ${}::timeframe, ${}::numeric)", step+1, step+2, step+3, step+4, step+5);
            placeholders.push(placeholder);
        }

        let slq_query = format!("INSERT INTO storage.lines (timestamp, exchange_pair, symbol, timeframe, value) VALUES {}", placeholders.join(", "));
        if !lines.is_empty() {
            match add_new_lines(&pool, &lines, &slq_query).await {
                Ok(_) => {
                    tracing::info!("Данные отправлены");
                    self.cache_aggregator_tx.try_send(CacheAggregatorCmd::AddLines { lines }).ok();
                    self.pending_lines.clear();
                },
                Err(e) => {
                    tracing::error!("Ошибка отправки батча: {e}");
                    self.pending_lines.clear();
                }
            };
        }
    }
}