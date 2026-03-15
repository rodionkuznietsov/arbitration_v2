use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use lru::LruCache;
use tokio::sync::{broadcast, mpsc, oneshot};
use crate::models::{orderbook::{BookEvent, Snapshot, SnapshotUi}, websocket::Symbol};

const PRICE_TICK: f64 = 900000000.0;

impl Snapshot {
    pub fn to_ui(&self, 
        depth: usize,
        last_price: f64,
    ) -> SnapshotUi {

        let mut a = self.a.iter()
            .filter(|(p, _)| (**p as f64) / PRICE_TICK >= last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / PRICE_TICK), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();
        a.reverse();

        let a_price = a
            .iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

        let a_price = match a_price {
            Some(a) => a.0,
            None => 0.0
        };

        let b = self.b.iter()
            .rev()
            .filter(|(p, _)| (**p as f64) / PRICE_TICK < a_price && (**p as f64) / PRICE_TICK <= last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / PRICE_TICK), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();


        let timestamp = self.timestamp;

        SnapshotUi {
            a,
            b,
            last_price,
            timestamp,
        }
    }
}

pub async fn parse_levels__(data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");
        let price_with_tick = (price * PRICE_TICK).round() as i64;

        values.insert(price_with_tick, volume);
    }

    values
}

pub struct BookData {
    pub snapshot: Option<Snapshot>,
    pub last_price: Option<f64>,
    pub volume24h: Option<f64>
}

pub enum ExchangeStoreCMD {
    Event(BookEvent),
    RegisterSymbol {
        symbol: Arc<Symbol>
    },
    GetBook { 
        symbol: Arc<Symbol>,
        reply: oneshot::Sender<SnapshotUi> 
    },
    GetQuote {
        symbol: Arc<Symbol>,
        reply: oneshot::Sender<(f64, f64)>
    },
    GetVolume {
        symbol: Arc<Symbol>,
        reply: oneshot::Sender<f64>
    }
} 

pub struct ExchangeStore {
    pub market_data: LruCache<Symbol, BookData>,
    pub rx: mpsc::Receiver<ExchangeStoreCMD>,
    pub book_updates: broadcast::Sender<String>,
}

impl ExchangeStore {
    pub fn new(rx: mpsc::Receiver<ExchangeStoreCMD>) -> Self {
        let cache_capacity = std::env::var("ORDERBOOK_CACHE_CAPACITY")
            .unwrap_or_else(|_| "1000".into())
            .parse::<usize>()
            .expect("ORDERBOOK_CACHE_CAPACITY must be a number");

        let (book_updates, _) = broadcast::channel(1);

        Self {
            market_data: LruCache::new(NonZeroUsize::new(cache_capacity).unwrap()),
            rx: rx,
            book_updates, 
        }
    }

    pub async fn set_data(mut self) {
        let mut last_version_id = 0;
        while let Some(cmd) = self.rx.recv().await {

            match cmd {
                ExchangeStoreCMD::RegisterSymbol { 
                    symbol 
                } => {
                    // Разобрать с normilize_symbol;
                    let symbol: String = symbol
                        .chars()
                        .filter(|c| *c != '_' && *c != '-')
                        .map(|c| c.to_ascii_lowercase())
                        .collect();

                    self.market_data.put(symbol, BookData { 
                        snapshot: None, 
                        last_price: None, 
                        volume24h: None
                    });
                },
                ExchangeStoreCMD::Event(event) => {
                    match event {
                        BookEvent::Snapshot { 
                            symbol, 
                            snapshot  
                        } => {
                            if let Some(data) = self.market_data.get_mut(&symbol) {
                                data.snapshot = Some(snapshot);
                            }
                            let _ = self.book_updates.send(symbol);
                        }
                        BookEvent::Delta { 
                            symbol, 
                            delta 
                        } => {
                            if let Some(data) = self.market_data.get_mut(&symbol) {
                                if let Some(snapshot) = &mut data.snapshot {
                                    match snapshot.last_update_id {
                                        Some(_) => {
                                            let from_version = delta.from_version.unwrap();
                                            let to_version = delta.to_version.unwrap();

                                            for (price, volume) in delta.a {
                                                if volume == 0.0 {
                                                    snapshot.a.remove(&price);
                                                } else {
                                                    if let Some(v) = snapshot.a.get_mut(&price) {
                                                        *v = volume;
                                                    } else {
                                                        snapshot.a.insert(price, volume);
                                                    }
                                                }
                                            }

                                            for (price, volume) in delta.b {
                                                if volume == 0.0 {
                                                    snapshot.b.remove(&price);
                                                } else {
                                                    if let Some(v) = snapshot.b.get_mut(&price) {
                                                        *v = volume;
                                                    } else {
                                                        snapshot.b.insert(price, volume);
                                                    }
                                                }
                                            }

                                            if last_version_id == 0 {
                                                continue;
                                            }

                                            if from_version != last_version_id {
                                                println!("[OrderBookManager]: Packet loss detected");
                                                continue;
                                            }
                                            last_version_id = to_version + 1;  
                                        }
                                        None => {
                                            for (price, volume) in delta.a {
                                                if volume == 0.0 {
                                                    snapshot.a.remove(&price);
                                                } else {
                                                    if let Some(v) = snapshot.a.get_mut(&price) {
                                                        *v = volume;
                                                    } else {
                                                        snapshot.a.insert(price, volume);
                                                    }
                                                }
                                            }

                                            for (price, volume) in delta.b {
                                                if volume == 0.0 {
                                                    snapshot.b.remove(&price);
                                                } else {
                                                    if let Some(v) = snapshot.b.get_mut(&price) {
                                                        *v = volume;
                                                    } else {
                                                        snapshot.b.insert(price, volume);
                                                    }
                                                }
                                            }
                                        }
                                    } 
                                }
                                let _ = self.book_updates.send(symbol);
                            }
                        },
                        BookEvent::TickerUpdate { 
                            symbol, 
                            last_price, 
                            volume 
                        } => {
                            if let Some(data) = self.market_data.get_mut(&symbol) {
                                data.last_price = Some(last_price);
                                data.volume24h = Some(volume);
                            }
                        },
                    }
                }
                ExchangeStoreCMD::GetBook { 
                    symbol, 
                    reply 
                } => {
                    if let Some(data) = self.market_data.get(&symbol.to_string()) {
                        let snapshot_ui = &data.snapshot;
                        if let Some(snapshot) = snapshot_ui {
                            if let Some(price) = data.last_price {
                                reply.send(snapshot.to_ui(6, price)).ok();
                            }
                        }
                    }
                }
                ExchangeStoreCMD::GetQuote { 
                    symbol ,
                    reply
                } => {                    
                    if let Some(data) = self.market_data.get(&symbol.to_string()) {
                        if let Some(snapshot) = &data.snapshot {
                            let best_ask = snapshot.a.iter()
                                .min_by_key(|x| x.0)
                                .map(|(price, _)| *price as f64 / PRICE_TICK);

                            let best_bid = snapshot.b.iter()
                                .max_by_key(|x| x.0)
                                .map(|(price, _)| *price as f64 / PRICE_TICK);

                            let best_ask = match best_ask {
                                Some(v) => v,
                                None => 0.0
                            };

                            let best_bid = match best_bid {
                                Some(v) => v,
                                None => 0.0
                            };

                            reply.send((best_ask, best_bid)).ok();
                        }
                    }
                },
                ExchangeStoreCMD::GetVolume { 
                    symbol, 
                    reply 
                } => {             
                    if let Some(data) = self.market_data.get(&symbol.to_string()) {
                        if let Some(volume) = data.volume24h {
                            reply.send(volume).ok();
                        }
                    }
                },
            }
        }
    }
}