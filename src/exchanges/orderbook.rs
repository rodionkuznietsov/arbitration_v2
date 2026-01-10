use std::{marker::PhantomData, sync::{Arc}};

use serde::{Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct BybitOrderbookLocal {
    pub snapshot: Snapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinanceOrderbookLocal {
    pub snapshot: Snapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalOrderBook {
    pub snapshot: Snapshot,
}

impl LocalOrderBook {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(
            Self {
                snapshot: Snapshot {
                    a: vec![],
                    b: vec![],
                    last_price: 0.0,
                },
            }
        ))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}
