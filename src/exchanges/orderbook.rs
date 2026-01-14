use std::{collections::{BTreeMap, HashMap}, sync::Arc};

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

    pub async fn to_hashmap(&self) -> HashMap<String, Snapshot> {
        self.books.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
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

        if let Some(mut snapshot) = self.books.get(&ticker.to_lowercase()) {
            let mut old_asks = &snapshot.a;
            let mut old_bids = &snapshot.b;

            // println!("{:#?}", old_asks)
            
            // let bids = snapshot.b.clone();

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
        let a = self.a.iter()
            .take(depth)
            .map(|(p, v)| (*p as f64 / 100.0, *v))
            .collect::<Vec<(f64, f64)>>();

        let b = self.b.iter()
            .take(depth)
            .map(|(p, v)| (*p as f64 / 100.0, *v))
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