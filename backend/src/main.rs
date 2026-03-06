use std::{time::Duration};
use tokio::sync::mpsc;

use crate::transport::{client_aggregator::{ClientAggregator}};

mod exchanges;
mod transport;
mod services;
mod storage;
mod models;

mod mexc_orderbook {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[tokio::main(flavor="multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();
    
    let storage_pool = storage::pool::create_pool().await;

    let (client_tx, client_rx) = mpsc::channel(1024);
    let client_aggregator = ClientAggregator::new(client_rx);

    tokio::spawn(client_aggregator.run());
    
    tokio::spawn({
        let client_tx = client_tx.clone();
        async move {
            services::market_manager::run_ws_exchanges(
                client_tx, storage_pool
            ).await;
        }
    });
    
    tokio::spawn({
        let client_tx = client_tx.clone();
        async move {
            transport::ws::connect_async(
                client_tx,
            ).await;
        }
    });

    loop {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
