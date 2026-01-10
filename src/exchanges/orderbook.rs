use serde::{Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct BybitOrderbookLocal {
    pub snapshot: Snapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct BinanceOrderbookLocal {
    pub snapshot: Snapshot,
}


#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}