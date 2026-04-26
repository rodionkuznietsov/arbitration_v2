use std::sync::Arc;

use crate::models::{orderbook::Snapshot, websocket::Symbol};

#[derive(Clone, Debug)]
/// <b>BookData</b> хранит данные `биржи`, получаемые с Websocket
pub struct BookData {
    pub snapshot: Option<Snapshot>,
    pub last_price: Option<f64>,
    pub volume24h: Option<f64>,
    pub symbol: Arc<Symbol>
}

impl BookData {
    pub fn new() -> Self {
        Self { 
            snapshot: None, 
            last_price: None, 
            volume24h: None,
            symbol: Arc::new(String::new())
        }
    }
}

#[derive(Debug)]
pub struct BookDataWithArc {
    pub snapshot: Option<Arc<Snapshot>>,
    pub last_price: Option<f64>,
    pub volume24h: Option<f64>
}