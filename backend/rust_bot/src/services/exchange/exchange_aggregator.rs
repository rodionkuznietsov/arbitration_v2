use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use lru::LruCache;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tokio::sync::{mpsc, watch};
use crate::models::{exchange::ExchangeType, exchange_aggregator::BookData, orderbook::{BookEvent, Delta, Snapshot, SnapshotUi}, websocket::Symbol};

impl Snapshot {
    pub fn to_ui(&self, 
        depth: usize,
        last_price: f64,
    ) -> SnapshotUi {

        let mut a = self.a.iter()
            .filter(|(p, _)| p.as_f64() >= last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some((p.as_f64(), *acc))
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
            .filter(|(p, _)| p.as_f64() < a_price && p.as_f64() <= last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some((p.as_f64(), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();

        SnapshotUi {
            a,
            b,
            last_price,
        }
    }
}

pub fn parse_levels__<'a>(data: Vec<Vec<&'a str>>) -> BTreeMap<Decimal, f64> {
    let mut values = BTreeMap::new();
    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");
        let decimal_price = rust_decimal::Decimal::from_f64(price).unwrap();

        values.insert(decimal_price, volume);
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
        reply: mpsc::Sender<watch::Receiver<Arc<BookData>>>
    },
    Default
} 

pub struct ExchangeStore {
    pub market_data: LruCache<Symbol, BookData>,
    pub rx: watch::Receiver<ExchangeStoreCMD>,
    pub register_channel_rx: mpsc::Receiver<ExchangeStoreCMD>,
    
    #[allow(unused)]
    id: ExchangeType,
    watch_tx: watch::Sender<Arc<BookData>>,
    watch_rx: watch::Receiver<Arc<BookData>>
}

impl ExchangeStore {
    pub fn new(
        rx: watch::Receiver<ExchangeStoreCMD>,
        register_channel_rx: mpsc::Receiver<ExchangeStoreCMD>,
        id: ExchangeType
    ) -> Self {
        let cache_capacity = std::env::var("ORDERBOOK_CACHE_CAPACITY")
            .unwrap_or_else(|_| "1000".into())
            .parse::<usize>()
            .expect("ORDERBOOK_CACHE_CAPACITY must be a number");

        let market_data = LruCache::new(NonZeroUsize::new(cache_capacity).unwrap());
        let (watch_tx, watch_rx) = watch::channel(Arc::new(BookData::new()));

        Self {
            market_data,

            rx: rx,
            watch_tx, watch_rx,
            register_channel_rx,

            id,
        }
    }

    pub async fn set_data(
        mut self,
    ) {
        let last_version_id = 0;

        loop {
            tokio::select! {
                Some(cmd) = self.register_channel_rx.recv() => {
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
                        
                            self.market_data.put(symbol.clone(), BookData::new());
                        },
                        ExchangeStoreCMD::Event(event) => {
                            match event {
                                BookEvent::Snapshot { 
                                    symbol, 
                                    snapshot  
                                } => {
                                    self.handle_snaphsot(symbol, snapshot);
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                },

                Ok(_) = self.rx.changed() => {
                    let cmd = self.rx.borrow().clone();

                    match cmd {
                        ExchangeStoreCMD::Default => {}
                        ExchangeStoreCMD::Event(event) => {
                            match event {
                                BookEvent::Snapshot { 
                                    symbol, snapshot  
                                } => {
                                    self.handle_snaphsot(symbol, snapshot);
                                }
                                BookEvent::Delta { 
                                    symbol, delta 
                                } => {
                                    self.handle_delta(symbol, delta, last_version_id);
                                },
                                BookEvent::TickerUpdate { 
                                    symbol, last_price, volume 
                                } => {
                                    self.ticker_updater(symbol, last_price, volume);
                                },
                            }
                        },
                        ExchangeStoreCMD::Subscribe { 
                            reply
                        } => {             
                            reply.send(self.watch_rx.clone()).await.ok();
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_snaphsot(
        &mut self,
        symbol: Symbol,
        snapshot: Snapshot
    ) {
        if let Some(data) = self.market_data.get_mut(&*symbol) {
            data.snapshot = Some(snapshot);
            data.symbol = Arc::new(symbol);
            let _ = self.watch_tx.send(Arc::new(data.to_owned()));
        }
    }

    fn handle_delta(
        &mut self,
        symbol: Symbol,
        delta: Delta,
        mut _last_version_id: u64
    ) {
        if let Some(data) = self.market_data.get_mut(&symbol) {
            if let Some(snapshot) = &mut data.snapshot {
                match snapshot.last_update_id {
                    Some(_) => {
                        let from_version = delta.from_version.unwrap();
                        let to_version = delta.to_version.unwrap();
                        _last_version_id = to_version + 1;  

                        if _last_version_id == 0 {   
                            return;
                        }
                        
                        Self::handle_delta_data(delta, snapshot);
                        
                        if from_version != _last_version_id {
                            tracing::error!("[OrderBookManager]: Packet loss detected");
                            return;
                        }
                        let _ = self.watch_tx.send(Arc::new(data.to_owned())); 
                    }
                    None => {
                        Self::handle_delta_data(delta, snapshot);
                        let _ = self.watch_tx.send(Arc::new(data.to_owned()));
                    }
                } 
            }
        }
    }

    fn handle_delta_data(
        delta: Delta,
        snapshot: &mut Snapshot
    ) {
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

    fn ticker_updater(
        &mut self,
        symbol: Symbol,
        last_price: f64,
        volume: f64
    ) {
        if let Some(data) = self.market_data.get_mut(&symbol) {
            data.last_price = Some(last_price);
            data.volume24h = Some(volume);
            let _ = self.watch_tx.send(Arc::new(data.to_owned()));
        }
    }
}