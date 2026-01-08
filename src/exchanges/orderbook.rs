use serde::{Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct OrderbookLocal {
    pub snapshot: Snapshot,
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64
}