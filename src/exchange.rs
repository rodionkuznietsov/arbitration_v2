use std::sync::Arc;
use tokio::sync::RwLock;

use crate::exchanges::{orderbook::LocalOrderBook, *};

pub enum Exchange {
    Bybit,
    Binance
}

impl Exchange {
    pub async fn connect(&self, ticker: &str, order_type: &str, book: Arc<RwLock<LocalOrderBook>>) {
        match self {
            Exchange::Binance => {
                binance::connect(ticker, order_type, book.clone()).await;
            }
            Exchange::Bybit => {
                bybit::connect(ticker, order_type, book.clone()).await;
            }
        }
    }
}