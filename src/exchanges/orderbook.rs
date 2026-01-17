use std::{collections::{BTreeMap}, sync::Arc};

use dashmap::DashMap;
use serde::{Serialize};
use tokio::sync::RwLock;

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

    pub fn set_last_price(&mut self, ticker: &str, last_price: f64) {
        if let Some(mut snapshot) = self.books.get_mut(&ticker.to_string()) {
            snapshot.last_price = last_price;
        } else {
            println!("[OrderBook] Ticker: {ticker} not found")
        }
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
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub last_price: f64,
}

impl Snapshot {
    pub fn to_ui(&self, depth: usize) -> SnapshotUi {
        let mut a = self.a.iter()
            .filter(|(p, _)| (**p as f64) / 100.0 > self.last_price)
            .map(|(p, v)| (*p as f64 / 100.0, *v))
            .take(depth)
            .collect::<Vec<(f64, f64)>>();
        a.reverse();

        let b = self.b.iter()
            .rev()
            .filter(|(p, _)| (**p as f64) / 100.00 <= self.last_price)
            .map(|(p, v)| (*p as f64 / 100.0, *v))
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