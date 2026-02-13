use std::{sync::Arc, time::Duration};
use crate::{transport::ws::ConnectedClient};

mod exchanges;
mod transport;
mod services;
mod storage;
mod models;

mod mexc_orderbook {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();
    
    let storage_pool = storage::pool::create_pool().await;

    let (client_tx, client_rx) = async_channel::unbounded::<ConnectedClient>();

    tokio::spawn({
        async move {
            services::market_manager::run_websockets(
                client_rx, storage_pool
            ).await;
        }
    });
    
    tokio::spawn({
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
