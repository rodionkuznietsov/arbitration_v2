use std::{collections::BTreeMap, num::NonZeroUsize};

use lru::LruCache;
use tokio::sync::mpsc;
use tracing::{warn};

use crate::models::orderbook::{BookEvent, Snapshot, SnapshotUi};

impl Snapshot {
    pub async fn to_ui(&self, depth: usize) -> SnapshotUi {
        let tick = 900000000.0;
        
        let mut a = self.a.iter()
            .filter(|(p, _)| (**p as f64) / tick >= self.last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / tick), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();
        a.reverse();

        let a_price = a
            .iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
            .unwrap()
            .0;

        let b = self.b.iter()
            .rev()
            .filter(|(p, _)| (**p as f64) / tick < a_price && (**p as f64) / tick <= self.last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / tick), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();

        let last_price = self.last_price;

        SnapshotUi {
            a,
            b,
            last_price
        }
    }
}

pub async fn parse_levels__(data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    let tick = 900000000.0;

    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");
        let price_with_tick = (price * tick).round() as i64;

        values.insert(price_with_tick, volume);
    }

    values
}

#[derive(Debug)]
pub enum OrderBookComand {
    Event(BookEvent),
    GetBook { 
        ticker: String,
        reply: mpsc::Sender<Option<SnapshotUi>> 
    },
} 

pub struct OrderBookManager {
    pub books: LruCache<String, Snapshot>,
    pub rx: mpsc::Receiver<OrderBookComand>
}

impl OrderBookManager {
    pub fn new(rx: mpsc::Receiver<OrderBookComand>) -> Self {
        let cache_capacity = std::env::var("ORDERBOOK_CACHE_CAPACITY")
            .unwrap_or_else(|_| "1000".into())
            .parse::<usize>()
            .expect("ORDERBOOK_CACHE_CAPACITY must be a number");
        
        Self {
            books: LruCache::new(NonZeroUsize::new(cache_capacity).unwrap()),
            rx: rx
        }
    }

    pub async fn set_data(mut self) {
        let mut last_version_id = 0;
        while let Some(cmd) = self.rx.recv().await {

            match cmd {
                OrderBookComand::Event(event) => {
                    match event {
                        BookEvent::Snapshot { ticker, snapshot  } => {
                            match self.books.get_mut(&ticker) {
                                Some(book) => {
                                    book.a = snapshot.a;
                                    book.b = snapshot.b;
                                }
                                None => {
                                    self.books.put(ticker.clone(), snapshot);
                                }
                            }
                        }
                        BookEvent::Delta { ticker, delta } => {
                            if let Some(snapshot) = self.books.get_mut(&ticker) {
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
                                            return;
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
                        }
                        BookEvent::Price { ticker, last_price } => {
                            if let Some(t) = self.books.get_mut(&ticker) {
                                t.last_price = last_price;
                            }
                        },
                    }
                }
                OrderBookComand::GetBook { ticker, reply } => {
                    if let Some(snapshot) = self.books.get(&format!("{}usdt", ticker)) {
                        let snapshot_ui = snapshot.to_ui(6).await;
                        match reply.send(Some(snapshot_ui)).await {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("[OrderBookManager]: {}", e)
                            }
                        }   
                    }
                }
            }
        }
    }
}