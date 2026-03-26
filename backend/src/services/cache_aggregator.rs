use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};
use tokio::sync::{mpsc};
use crate::{models::{aggregator::{KeyMarketType, KeyPair}, line::Line}, storage::line_storage::{get_all_spread_history}};

const MAX_LINES: usize = 100;

#[derive(Debug)]
pub enum CacheAggregatorCmd {
    AddLines {
        lines: Vec<(Line, KeyPair)>
    },
    LinesHistory {
        reply: mpsc::Sender<HashMap<KeyMarketType, VecDeque<Line>>>
    }
}

pub struct CacheAggregator {
    cache_lines: HashMap<KeyMarketType, VecDeque<Line>>,
    initialization_keys: HashSet<KeyMarketType>,

    cache_aggregator_rx: mpsc::Receiver<Arc<CacheAggregatorCmd>>,
    pool: sqlx::PgPool,
}

impl CacheAggregator {
    pub fn new(
        cache_aggregator_rx: mpsc::Receiver<Arc<CacheAggregatorCmd>>,
        pool: sqlx::PgPool,
    ) -> Self {
        Self { 
            cache_lines: HashMap::new(),
            initialization_keys: HashSet::new(),
            
            cache_aggregator_rx,

            pool,
        }
    }

    pub async fn run(
        mut self,
    ) {
        loop {
            if let Some(cmd) = self.cache_aggregator_rx.recv().await {
                match cmd.as_ref() {
                    CacheAggregatorCmd::AddLines { 
                        lines
                    } => {
                        for (line, key) in lines {
                            let long_deque = self.cache_lines
                                .entry(key.long_market_type.clone())
                                .or_insert_with(VecDeque::new);

                            long_deque.push_back(line.clone());

                            if long_deque.len() > MAX_LINES {
                                long_deque.pop_front();
                            }

                            long_deque.make_contiguous().sort_by(|x, y| x.timestamp.cmp(&y.timestamp));

                            let short_deque = self.cache_lines
                                .entry(key.short_market_type.clone())
                                .or_insert_with(VecDeque::new);

                            short_deque.push_back(line.clone());

                            if short_deque.len() > MAX_LINES {
                                short_deque.pop_front();
                            }

                            short_deque.make_contiguous().sort_by(|x, y| x.timestamp.cmp(&y.timestamp));
                        }
                    },
                    CacheAggregatorCmd::LinesHistory {
                        reply
                    } => {
                        if self.initialization_keys.is_empty() {
                            let history = get_all_spread_history(&self.pool).await;
                            if let Ok(lines) = history {
                                for ((long_ex, short_ex, symbol), lines) in lines {
                                    self.cache_lines.insert(KeyMarketType { 
                                        long_exchange: long_ex, 
                                        short_exchange: short_ex, 
                                        symbol: Arc::new(symbol.clone()) 
                                    }, lines.clone());


                                    self.initialization_keys.insert(
                                        KeyMarketType::new(
                                            long_ex, 
                                            short_ex, 
                                            Arc::new(symbol)
                                        )
                                    );
                                }
                            }
                        } 
                        let _ = reply.send(self.cache_lines.clone()).await;
                    },
                }
            }
        }
    }
}