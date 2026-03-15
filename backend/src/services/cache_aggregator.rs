use std::{collections::{HashMap, VecDeque}};
use tokio::sync::{mpsc};

use crate::{models::{aggregator::KeyPair, line::{Line, MarketType}}, storage::line_storage::get_spread_history};

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
    cache_lines: HashMap<KeyPair, VecDeque<Line>>,
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
                            let deque = self.cache_lines
                                .entry(key)
                                .or_insert_with(VecDeque::new);

                            deque.push_back(line);

                            if deque.len() > MAX_LINES {
                                deque.pop_front();
                            }
                        }
                    },
                    CacheAggregatorCmd::RemovePair { 
                        key
                    } => {

                    },
                    CacheAggregatorCmd::GetLinesHistory { 
                        key,
                        reply
                    } => {
                        if let Some(lines) = self.cache_lines.get(&key).filter(|v| v.len() >= 100) {
                            println!("VECC{:?}", lines);
                        } else {
                            let long_exchange_pair = format!("{}/{}", key.long_exchange, key.short_exchange);
                            let lines = get_spread_history(&self.pool, &key.symbol, &long_exchange_pair).await.ok();
                            if let Some(lines) = lines {
                                reply.send((lines, MarketType::Long)).await.ok();
                            }

                            let short_exchange_pair = format!("{}/{}", key.short_exchange, key.long_exchange);
                            let lines = get_spread_history(&self.pool, &key.symbol, &short_exchange_pair).await.ok();
                            if let Some(lines) = lines {
                                reply.send((lines, MarketType::Short)).await.ok();
                            }
                        }
                    },
                }
            }
        }
    }
}