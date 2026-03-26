use std::{collections::HashMap, sync::Arc, time::{Duration}};
use chrono::{Timelike, Utc, Duration as ChronoDuration};
use tokio::{sync::{mpsc}, time::{Instant as TokioInstant, interval_at}};
use crate::{models::{aggregator::{KeyMarketType, KeyPair, SpreadPair}, exchange::ExchangeType, exchange_aggregator::BookData, websocket::Symbol}, services::data_mapping::DataMappingCmd};

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

#[derive(Clone, Copy, Debug)]
pub struct Quote {
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone)]
pub struct ExchangeBookData {
    pub data: Option<Arc<BookData>>
}

/// <b>DataAggregator</b> Обьединяет данные с разных бирж
pub struct DataAggregator {
    pending_lines: HashMap<KeyPair, SpreadPair>,
    markets: HashMap<Arc<Symbol>, HashMap<ExchangeType, ExchangeBookData>>,

    rx: mpsc::Receiver<DataAggregatorCmd>,
    data_mapping_tx: mpsc::Sender<DataMappingCmd>,

    pool: sqlx::PgPool,
}

impl DataAggregator {
    pub fn new(
        aggregator_rx: mpsc::Receiver<DataAggregatorCmd>,
        data_mapping_tx: mpsc::Sender<DataMappingCmd>,
        pool: sqlx::PgPool,
    ) -> Self {
        let markets = HashMap::new();
        Self {
            pending_lines: HashMap::new(),
            markets,

            rx: aggregator_rx,
            data_mapping_tx,

            pool,
        }
    }
    
    pub async fn run(
        mut self, 
        data_mapping_tx: mpsc::Sender<DataMappingCmd>,
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

        loop {
            tokio::select! {
                Some(cmd) = self.rx.recv() => {
                    self.handle_command(cmd).await;
                    
                    // data_mapping_tx.send_timeout(
                    //     DataMappingCmd::ExchangesDataToJsonPair(
                    //         self.markets.clone()
                    //     ), 
                    //     Duration::from_millis(ORDERBOOK_DELAY)
                    // ).await.ok();
                }

                _ = interval.tick() => {
                    self.db_writer().await;
                },

                Some((key_pair, spread)) = spread_rx.recv() => {                    
                    self.pending_lines.insert(
                        key_pair, 
                        spread
                    );
                }
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
                data
            } => {
                if let Some(exchanges) = self.markets.get_mut(&symbol) {
                    if let Some(old_data) = exchanges.get_mut(&exchange_id) {
                        old_data.data = Some(data);
                    }

                    let exchange_data: Vec<(ExchangeType, Arc<Symbol>, Arc<BookData>)> = exchanges
                        .iter()
                        .filter_map(|(ex_id, data)| {
                            data.data.as_ref().map(|arc| (*ex_id, symbol.clone(), arc.clone()))
                        })
                        .collect();

                    self.data_mapping_tx.send_timeout(
                        DataMappingCmd::ExchangesDataToJsonPair(
                            exchange_data
                        ), 
                        Duration::from_millis(10)
                    ).await.ok();
                }
            }
        }
        
    }

    // async fn send_payload_to_client_aggregator(
    //     &mut self,
    //     spread_tx: mpsc::Sender<(KeyPair, SpreadPair)>,
    //     symbol: Arc<Symbol>
    // ) {
    //     let data = self.markets.get(&symbol);
    //     if let Some(exchanges) = data {
    //         for (i, (long_ex, long_data)) in exchanges.iter().enumerate() {
    //             for (short_ex, short_data) in exchanges.iter().skip(i+1) {
    //                 let long_book = &long_data.snapshot;
    //                 let short_book = &short_data.snapshot;

    //                 if let (Some(long), Some(short)) = (long_book, short_book) {

    //                 }

    //                 let long_volume = long_data.volume24h;
    //                 let short_volume = short_data.volume24h;

    //                 if let (
    //                     Some(long_volume), 
    //                     Some(short_volume)
    //                 ) = (
    //                     long_volume, 
    //                     short_volume
    //                 ) {

    //                 }

    //                 if let (
    //                     Some(long_quote), 
    //                     Some(short_quote)
    //                 ) = (
    //                     long_data.quote, 
    //                     short_data.quote
    //                 ) {
    //                     self.calculate_spread(
    //                         spread_tx.clone(),
    //                         symbol.clone(),
    //                         *long_ex,
    //                         long_quote,
    //                         *short_ex,
    //                         short_quote,
    //                     ).await;
    //                 }
    //             }
    //         }
    //     }
    // }

    async fn calculate_spread(
        &self,
        spread_tx: mpsc::Sender<(KeyPair, SpreadPair)>,

        symbol: Arc<Symbol>,

        long_exchange: ExchangeType,
        long_quote: Quote,

        short_exchange: ExchangeType,
        short_quote: Quote,
    ) {
        let mid_in_price = (short_quote.bid + long_quote.ask) / 2.0;
        let spread_in_percent = (short_quote.bid - long_quote.ask) / mid_in_price * 100.0;
        
        let mid_out_price = (long_quote.ask + short_quote.bid) / 2.0;
        let spread_out_percent = (long_quote.ask - short_quote.bid) / mid_out_price * 100.0;

        let now = Utc::now();
        let timestamp = now.timestamp() - (now.timestamp() % 60);

        // spread_tx.send_timeout(
        //     (
        //         KeyPair::new(
        //             KeyMarketType::new(long_exchange, short_exchange, symbol.clone()),
        //             KeyMarketType::new(short_exchange, long_exchange, symbol.clone()),
        //         ),
        //         SpreadPair::new(
        //             spread_in_percent, 
        //             spread_out_percent, 
        //             timestamp
        //         )
        //     ),
        //     Duration::from_millis(SPREAD_DELAY)
        // ).await.ok();

        // let long_key = ChannelSubscription::Chart { 
        //     long_exchange, 
        //     short_exchange, 
        //     ticker: symbol.clone()
        // };

        // let short_key = ChannelSubscription::Chart { 
        //     long_exchange: short_exchange, 
        //     short_exchange: long_exchange, 
        //     ticker: symbol.clone()
        // };

        // manager_transmitter_tx.send_timeout(
        //     ManagerTransmitterCmd::Notify(
        //         NotifyEvent::Payload(long_key, long_payload, short_key, short_payload),
        //     ),
        //     Duration::from_millis(CHART_DELAY)
        // ).await.ok();
    }

    /// Обрабатывает pending_lines и сохраняет в базу данных
    async fn db_writer(
        &mut self,
    ) {
        // let pool = self.pool.clone();
        
        // let mut placeholders: Vec<String> = Vec::new();
        // let mut lines = Vec::new();

        // for (i, (key_pair, spread_pair)) in self.pending_lines.iter().enumerate() {
            
        //     let step = i*5;            

        //     let long_line = Line::new(
        //         key_pair.long_market_type.long_exchange, 
        //         key_pair.long_market_type.short_exchange, 
        //         key_pair.long_market_type.symbol.to_string(), 
        //         spread_pair.long_spread, 
        //         TimeFrame::One, 
        //         spread_pair.timestamp * 1000
        //     );

        //     let short_line = Line::new(
        //         key_pair.short_market_type.long_exchange, 
        //         key_pair.short_market_type.short_exchange, 
        //         key_pair.short_market_type.symbol.to_string(), 
        //         spread_pair.short_spread, 
        //         TimeFrame::One, 
        //         spread_pair.timestamp * 1000
        //     );

        //     lines.push((long_line, key_pair.clone()));
        //     lines.push((short_line, key_pair.clone()));

        //     // Создаем плейсхолдеры для sql_query
        //     let placeholder = format!("(${}, ${}, ${}, ${}::timeframe, ${}::numeric)", step+1, step+2, step+3, step+4, step+5);
        //     placeholders.push(placeholder);
        // }

        // let slq_query = format!("INSERT INTO storage.lines (timestamp, exchange_pair, symbol, timeframe, value) VALUES {}", placeholders.join(", "));
        // if !lines.is_empty() {
        //     match add_new_lines(&pool, &lines, &slq_query).await {
        //         Ok(_) => {
        //             tracing::info!("Данные отправлены");
        //             manager_transmitter_tx.send_timeout(
        //                 ManagerTransmitterCmd::Notify(
        //                     NotifyEvent::Cache(
        //                         CacheAggregatorCmd::AddLines { lines }
        //                     )
        //                 ),
        //                 Duration::from_millis(CHART_DELAY)
        //             ).await.ok();
        //             self.pending_lines.clear();
        //         },
        //         Err(e) => {
        //             tracing::error!("Ошибка отправки батча: {e}");
        //             self.pending_lines.clear();
        //         }
        //     };
        // }
    }
}