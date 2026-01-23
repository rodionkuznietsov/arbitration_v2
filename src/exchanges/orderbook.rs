use std::{collections::{BTreeMap, HashMap}, sync::Arc};

use dashmap::DashMap;
use serde::{Serialize};
use tokio::sync::{RwLock, mpsc, oneshot};

type Ticker = String;

#[derive(Clone, Debug)]
pub enum OrderType {
    Long, 
    Short
}

#[derive(Debug, Clone)]
pub struct LocalOrderBook {
    pub books: DashMap<Ticker, Snapshot>
}

impl LocalOrderBook {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(
            Self {
                books: DashMap::new()
            }
        ))
    }

    pub async fn set_last_price(&mut self, ticker: &str, last_price: f64) {
        if self.books.contains_key(ticker) {
            if let Some(mut snapshot) = self.books.get_mut(&ticker.to_string()) {
                snapshot.last_price = last_price;
            } else {
                println!("[OrderBook] Ticker: {ticker} not found")
            }
        }
    }

    pub async fn parse_levels(&self, data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
        let mut values = BTreeMap::new();
        let tick = 1000000.0;

        for vec in data {
            let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
            let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");

            let price_with_tick = (price * tick).round() as i64;

            values.insert(price_with_tick, volume);
        }

        values
    }

    pub async fn apply_snapshot_updates(
        &mut self, 
        ticker: &str, 
        asks: BTreeMap<i64, f64>,
        bids: BTreeMap<i64, f64>,
    ) {

        if let Some(mut snapshot) = self.books.get_mut(&ticker.to_lowercase()) {
            for (price, volume) in asks {
                if volume == 0.0 {
                    snapshot.a.remove(&price);
                } else {
                    snapshot.a.insert(price, volume);
                }
            }

            for (price, volume) in bids {
                if volume == 0.0 {
                    snapshot.b.remove(&price);
                } else {
                    snapshot.b.insert(price, volume);
                }
            }
            
        } else {
            println!("[OrderBook] Ticker: {ticker} not found")
        }
    }

    pub async fn apply_or_add_snapshot(
        &self, 
        ticker: &str, 
        asks: BTreeMap<i64, f64>, 
        bids: BTreeMap<i64, f64>
    ) {
        if !self.books.contains_key(ticker) {
            self.books.insert(ticker.to_lowercase(), Snapshot {
                a: asks,
                b: bids,
                last_price: 0.0
            });
        } else {
            if let Some(mut snapshot) = self.books.get_mut(ticker) {
                snapshot.a = asks;
                snapshot.b = bids;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub last_price: f64,
}

impl Snapshot {
    pub async fn to_ui(&self, depth: usize) -> SnapshotUi {
        let tick = 1000000.00;
        let mut a = self.a.iter()
            .filter(|(p, _)| (**p as f64) / tick > self.last_price)
            .map(|(p, v)| (*p as f64 / tick, *v))
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
            .filter(|(p, _)| (**p as f64) / tick < a_price)
            .map(|(p, v)| (*p as f64 / tick, *v))
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

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotUi {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}

pub async fn parse_levels__(data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    let tick = 1000000.0;

    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");

        let price_with_tick = (price * tick).round() as i64;

        values.insert(price_with_tick, volume);
    }

    values
}

pub struct OrderBookView {
    snapshot: Snapshot
}

pub enum BookEvent {
    Snapshot { ticker: String, snapshot: Snapshot },
    Price { ticker: String, last_price: f64 },
    GetBook { ticker: String, reply: mpsc::Sender<Option<SnapshotUi>> }
}

#[derive(Clone)]
pub struct OrderBookManager {
    pub books: HashMap<String, Snapshot>,
    pub rx: async_channel::Receiver<BookEvent>
}

impl OrderBookManager {
    pub async fn set_data(mut self) {
        while let Ok(event) = self.rx.clone().recv().await {
            match event {
                BookEvent::Snapshot { ticker, snapshot  } => {
                    let snapshot_ = snapshot.clone();
                    self.books
                        .entry(ticker.clone())
                        .and_modify(|book| {
                            book.a = snapshot_.a;
                            book.b = snapshot_.b;
                        })
                        .or_insert_with(|| Snapshot { a: snapshot.a, b: snapshot.b, last_price: 0.0 });
                }
                BookEvent::Price { ticker, last_price } => {
                    if let Some(t) = self.books.get_mut(&ticker) {
                        t.last_price = last_price;
                    }
                },
                BookEvent::GetBook { ticker, reply } => {
                    if let Some(snapshot) = self.books.get(&format!("{}usdt", ticker)) {   
                        let snapshot_ui = snapshot.to_ui(6).await;
                        match reply.send(Some(snapshot_ui)).await {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{}", format!("[OrderBookManager]: {}", e))
                            }
                        }   
                    }
                }
            }
        }
    }
}