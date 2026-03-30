use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};
use tokio::sync::{Mutex, RwLock, mpsc};
use crate::{models::{aggregator::KeyMarketType, exchange::ExchangeType, line::Line, websocket::Symbol}, services::data_mapping::DataMappingCmd, storage::line_storage::{get_spread_history}};

const MAX_LINES: usize = 100;

#[derive(Debug)]
pub enum CacheAggregatorCmd {
    AddLines {
        lines: Vec<(Line, (ExchangeType, ExchangeType, Arc<std::string::String>))>
    },
    LinesHistory {
        reply: mpsc::Sender<HashMap<(ExchangeType, Arc<Symbol>), VecDeque<Line>>>
    },
    InitAllLines {
        key: KeyMarketType,
    }
}

pub struct CacheAggregator {
    cache_lines: Arc<Mutex<Arc<HashMap<(ExchangeType, ExchangeType), HashMap<Arc<Symbol>, Arc<RwLock<VecDeque<Line>>>>>>>>,
    initialization_keys: HashSet<KeyMarketType>,

    cache_aggregator_rx: mpsc::Receiver<Arc<CacheAggregatorCmd>>,
    data_mapping_tx: mpsc::Sender<DataMappingCmd>,

    pool: sqlx::PgPool,
}

impl CacheAggregator {
    pub fn new(
        cache_aggregator_rx: mpsc::Receiver<Arc<CacheAggregatorCmd>>,
        data_mapping_tx: mpsc::Sender<DataMappingCmd>,

        pool: sqlx::PgPool,
    ) -> Self {
        Self { 
            cache_lines: Arc::new(Mutex::new(Arc::new(HashMap::new()))),
            initialization_keys: HashSet::new(),
            
            cache_aggregator_rx,
            data_mapping_tx,

            pool,
        }
    }

    pub async fn run(
        mut self,
    ) {        
        while let Some(cmd) = self.cache_aggregator_rx.recv().await {
            match cmd.as_ref() {
                CacheAggregatorCmd::AddLines { 
                    lines
                } => {
                    let mut lock = self.cache_lines.lock().await;
                    let mut new_map = (*lock).as_ref().clone();

                    for (line, (long_exchange, short_exchange, symbol)) in lines.into_iter() {                        
                        let map = new_map
                            .entry((*long_exchange, *short_exchange))
                            .or_insert_with(HashMap::new);
                        
                        let deque = map
                            .entry(symbol.clone())
                            .or_insert_with(|| Arc::new(RwLock::new(VecDeque::new())));

                        let mut dq = deque.write().await;

                        let pos = dq
                            .iter()
                            .position(|l| l.timestamp > line.timestamp)
                            .unwrap_or(dq.len());

                        dq.insert(pos, line.clone());

                        if dq.len() > MAX_LINES {
                            dq.pop_front();
                        }
                    }

                    *lock = Arc::new(new_map);
                },
                CacheAggregatorCmd::LinesHistory {
                    reply
                } => {
                    // if self.initialization_keys.is_empty() {
                    //     let history = get_all_spread_history(&self.pool).await;
                    //     if let Ok(lines) = history {
                    //         for ((long_ex, short_ex, symbol), lines) in lines {
                    //             self.cache_lines.insert(KeyMarketType { 
                    //                 long_exchange: long_ex, 
                    //                 short_exchange: short_ex, 
                    //                 symbol: Arc::new(symbol.clone()) 
                    //             }, lines.clone());


                    //             self.initialization_keys.insert(
                    //                 KeyMarketType::new(
                    //                     long_ex, 
                    //                     short_ex, 
                    //                     Arc::new(symbol)
                    //                 )
                    //             );
                    //         }
                    //     }
                    // } 
                    // let _ = reply.send(self.cache_lines.clone()).await;
                },
                CacheAggregatorCmd::InitAllLines { 
                    key,
                } => {
                    if !self.initialization_keys.contains(&key) {
                        let result = get_spread_history(&self.pool, &key.symbol, key.long_exchange, key.short_exchange).await;
                        if let Ok(lines) = result {
                            if !lines.is_empty() {
                                self.data_mapping_tx.send(DataMappingCmd::LinesFromDbToJsonPair(lines.clone())).await.ok();
                                self.initialization_keys.insert(key.clone());
                                
                                let mut lock = self.cache_lines.lock().await;
                                let mut new_map = (*lock).as_ref().clone();
                                
                                if let Some(lines) = lines.get(&(key.long_exchange, key.short_exchange, key.symbol.clone())) {
                                    for line in lines {
                                        let map = new_map
                                            .entry((key.long_exchange, key.short_exchange))
                                            .or_insert_with(HashMap::new);

                                        let deque = map
                                            .entry(key.symbol.clone())
                                            .or_insert_with(|| Arc::new(RwLock::new(VecDeque::new())));

                                        let mut dq = deque.write().await;
                                    
                                        let pos = dq
                                            .iter()
                                            .position(|l| l.timestamp > line.timestamp)
                                            .unwrap_or(dq.len());

                                        dq.insert(pos, line.clone());

                                        if dq.len() > MAX_LINES {
                                            dq.pop_front();
                                        }
                                    }
                                    *lock = Arc::new(new_map);
                                }
                            }
                        }
                    } else {
                        let cache_lines = self.cache_lines.lock().await.clone();
                        if let (
                            Some(long_map), 
                            Some(short_map), 
                        ) = (
                            cache_lines.get(&(key.long_exchange, key.short_exchange)), 
                            cache_lines.get(&(key.short_exchange, key.long_exchange)), 
                        ) {
                            if let (
                                Some(long_data), 
                                Some(short_data)
                            ) = (
                                short_map.get(&key.symbol), 
                                long_map.get(&key.symbol)
                            ) {
                                self.data_mapping_tx.send(DataMappingCmd::LinesToJsonPair(
                                    long_data.clone(), 
                                    short_data.clone(),
                                    key.symbol.clone(),
                                    key.long_exchange,
                                    key.short_exchange
                                )).await.ok();
                            }
                        }
                    }
                }
            }
        }
    }
}