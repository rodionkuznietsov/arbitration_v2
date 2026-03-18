use std::{collections::{HashMap, HashSet, VecDeque}};
use tokio::sync::{mpsc};
use tracing::{error, info};

use crate::{models::{aggregator::{KeyMarketType, KeyPair}, line::{Line, MarketType}}, storage::line_storage::get_spread_history};

const MAX_LINES: usize = 100;

pub enum CacheAggregatorCmd {
    AddLines {
        lines: Vec<(Line, KeyPair)>
    },
    RemovePair {
        key: KeyPair
    },
    GetLinesHistory {
        key: KeyPair,
        reply: mpsc::Sender<(VecDeque<Line>, MarketType)>
    }
}

pub struct CacheAggregator {
    cache_lines: HashMap<KeyMarketType, VecDeque<Line>>,
    initialization_keys: HashSet<KeyMarketType>,

    cache_aggregator_rx: mpsc::Receiver<CacheAggregatorCmd>,
    pool: sqlx::PgPool,
}

impl CacheAggregator {
    pub fn new(
        cache_aggregator_rx: mpsc::Receiver<CacheAggregatorCmd>,
        pool: sqlx::PgPool,
    ) -> Self {
        Self { 
            cache_lines: HashMap::new(),
            initialization_keys: HashSet::new(),
            
            cache_aggregator_rx,
            pool
        }
    }

    pub async fn run(
        mut self,
    ) {
        loop {
            if let Some(cmd) = self.cache_aggregator_rx.recv().await {
                match cmd {
                    CacheAggregatorCmd::AddLines { 
                        lines
                    } => {
                        for (line, key) in lines {
                            let long_deque = self.cache_lines
                                .entry(key.long_market_type)
                                .or_insert_with(VecDeque::new);

                            long_deque.push_back(line.clone());

                            if long_deque.len() > MAX_LINES {
                                long_deque.pop_front();
                            }

                            long_deque.make_contiguous().sort_by(|x, y| x.timestamp.cmp(&y.timestamp));

                            let short_deque = self.cache_lines
                                .entry(key.short_market_type)
                                .or_insert_with(VecDeque::new);

                            short_deque.push_back(line);

                            if short_deque.len() > MAX_LINES {
                                short_deque.pop_front();
                            }

                            short_deque.make_contiguous().sort_by(|x, y| x.timestamp.cmp(&y.timestamp));
                        }
                    },
                    CacheAggregatorCmd::RemovePair { 
                        key
                    } => {
                        if let Some(_) = self.cache_lines.remove(&key.long_market_type) {
                            self.initialization_keys.remove(&key.long_market_type);
                        }

                        if let Some(_) = self.cache_lines.remove(&key.short_market_type) {
                            self.initialization_keys.remove(&key.short_market_type);
                        }
                    },
                    CacheAggregatorCmd::GetLinesHistory { 
                        key,
                        reply
                    } => {
                        // Проверяем инициализированы ли данные по этому ключу
                        if self.initialization_keys.contains(&key.long_market_type.clone()) {
                            if let Some(lines) = self.cache_lines.get_mut(&key.long_market_type) {
                                reply.send((lines.clone(), MarketType::Long)).await.ok();
                            }
                        } else {
                            let exchange_pair = format!("{}/{}", key.long_market_type.long_exchange, key.long_market_type.short_exchange);
                            let lines = match get_spread_history(&self.pool, &key.long_market_type.symbol, &exchange_pair).await {
                                Ok(l) => Some(l),
                                Err(e) => {
                                    error!("{:?} -> {e}", key.long_market_type);
                                    None
                                }
                            };
                            
                            if let Some(lines) = lines {
                                info!("Инициализация данных для ключа: {:?}", key.long_market_type);

                                self.cache_lines.insert(key.long_market_type.clone(), lines.clone());
                                self.initialization_keys.insert(key.long_market_type);
                                reply.send((lines, MarketType::Long)).await.ok();
                            }
                        }

                        // Проверяем инициализированы ли данные по этому ключу
                        if self.initialization_keys.contains(&key.short_market_type.clone()) {
                            if let Some(lines) = self.cache_lines.get_mut(&key.short_market_type) {
                                reply.send((lines.clone(), MarketType::Short)).await.ok();
                            }
                        } else {
                            let exchange_pair = format!("{}/{}", key.short_market_type.long_exchange, key.short_market_type.short_exchange);
                            let lines = match get_spread_history(&self.pool, &key.short_market_type.symbol, &exchange_pair).await {
                                Ok(l) => Some(l),
                                Err(e) => {
                                    error!("{:?} -> {e}", key.short_market_type);
                                    None
                                }
                            };
                            
                            if let Some(lines) = lines {
                                info!("Инициализация данных для ключа: {:?}", key.short_market_type);

                                self.cache_lines.insert(key.short_market_type.clone(), lines.clone());
                                self.initialization_keys.insert(key.short_market_type);
                                reply.send((lines, MarketType::Short)).await.ok();
                            }
                        }
                    },
                }
            }
        }
    }
}