use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use lru::LruCache;
use tokio::sync::{mpsc, watch};
use crate::models::{exchange::ExchangeType, exchange_aggregator::BookData, orderbook::{BookEvent, Snapshot, SnapshotUi}, websocket::Symbol};

pub const PRICE_TICK: f64 = 900000000.0;

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

pub fn parse_levels__(data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");
        let price_with_tick = (price * PRICE_TICK).round() as i64;

        values.insert(price_with_tick, volume);
    }

    values
}

#[derive(Debug, Clone)]
pub enum ExchangeStoreCMD {
    Event(BookEvent),
    RegisterSymbol {
        symbol: Arc<Symbol>
    },
    Subscribe {
        reply: mpsc::Sender<watch::Receiver<(Arc<Symbol>, Arc<BookData>)>>
    },
    Default
} 

pub struct ExchangeStore {
    pub market_data: LruCache<Symbol, BookData>,
    pub rx: watch::Receiver<ExchangeStoreCMD>,
    #[allow(unused)]
    id: ExchangeType,
    watch_tx: watch::Sender<(Arc<Symbol>, Arc<BookData>)>,
    watch_rx: watch::Receiver<(Arc<Symbol>, Arc<BookData>)>
}

impl ExchangeStore {
    pub fn new(
        rx: watch::Receiver<ExchangeStoreCMD>,
        id: ExchangeType
    ) -> Self {
        let cache_capacity = std::env::var("ORDERBOOK_CACHE_CAPACITY")
            .unwrap_or_else(|_| "1000".into())
            .parse::<usize>()
            .expect("ORDERBOOK_CACHE_CAPACITY must be a number");

        let market_data = LruCache::new(NonZeroUsize::new(cache_capacity).unwrap());
        let (watch_tx, watch_rx) = watch::channel((Arc::new(Symbol::new()), Arc::new(BookData::new())));

        Self {
            market_data,
            rx: rx,
            watch_tx, watch_rx,

            id,
        }
    }

    pub async fn set_data(
        mut self,
    ) {
        let mut last_version_id = 0;
        while let Ok(_) = self.rx.changed().await {
            let cmd = self.rx.borrow().clone();

            match cmd {
                ExchangeStoreCMD::Default => {

                }
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
                            if let Some(data) = self.market_data.get_mut(&*symbol) {
                                data.snapshot = Some(snapshot);
                                let _ = self.watch_tx.send((Arc::new(symbol), Arc::new(data.to_owned())));
                            }
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
                                            let _ = self.watch_tx.send((Arc::new(symbol), Arc::new(data.to_owned())));
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
                                            let _ = self.watch_tx.send((Arc::new(symbol), Arc::new(data.to_owned())));
                                        }
                                    } 
                                }
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
                                let _ = self.watch_tx.send((Arc::new(symbol), Arc::new(data.to_owned())));
                            }
                        },
                    }
                },
                ExchangeStoreCMD::Subscribe { 
                    reply
                } => {             
                    reply.try_send(self.watch_rx.clone()).ok();
                },
            }
        }
    }
}