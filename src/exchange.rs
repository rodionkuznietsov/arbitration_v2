use std::sync::Arc;
use tokio::sync::RwLock;

use crate::exchanges::{orderbook::LocalOrderBook, *};

pub enum Exchange {
    Bybit,
    Binance,
    KuCoin,
    BinX
}

impl Exchange {
    pub async fn connect(&self, ticker: &str, channel_type: &str, book: Arc<RwLock<LocalOrderBook>>) {
        match self {
            Exchange::Binance => {
                binance::connect(ticker, channel_type, book.clone()).await;
            }
            Exchange::Bybit => {
                bybit::connect(ticker, channel_type, book.clone()).await;
            }
            Exchange::KuCoin => {
                kucoin::connect(ticker, channel_type, book.clone()).await;
            }
            Exchange::BinX => {
                binx::connect(ticker, channel_type, book.clone()).await;
            }
        }
    }
}