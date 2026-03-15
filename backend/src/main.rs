use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, watch};

use crate::{services::{cache_aggregator::{CacheAggregator, CacheAggregatorCmd}, data_aggregator::{DataAggregator, DataAggregatorCmd}}, transport::client_aggregator::{ClientAggregator, ClientAggregatorCmd}};

mod exchanges;
mod transport;
mod services;
mod storage;
mod models;
mod adapters;

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

    // Запускаем агррегаторы
    let (cache_aggregator_tx, cache_aggregator_rx) = mpsc::channel::<CacheAggregatorCmd>(1);
    let cache_aggregator = CacheAggregator::new(
        cache_aggregator_rx, 
        storage_pool.clone()
    );
    tokio::spawn(cache_aggregator.run());

    // Канал для получения данных с data aggregator
    let (client_aggregator_watch_tx, client_aggregator_watch_rx) = watch::channel(
        Arc::new(ClientAggregatorCmd::Init)
    );

    // Канал для приёма команд от пользователя
    let (client_aggregator_tx, client_aggregator_rx) = mpsc::channel::<ClientAggregatorCmd>(32);

    let client_aggregator = ClientAggregator::new(
        client_aggregator_watch_rx,
        client_aggregator_rx,
        cache_aggregator_tx.clone(),
    );
    tokio::spawn(client_aggregator.run());
    
    let (data_aggregator_tx, data_aggregator_rx) = mpsc::channel::<DataAggregatorCmd>(1);
    let data_aggregator = DataAggregator::new(
        data_aggregator_rx, 
        cache_aggregator_tx.clone(),
        storage_pool.clone(),
    );
    tokio::spawn(data_aggregator.run(client_aggregator_watch_tx.clone()));

    tokio::spawn({
        let data_aggregator_tx = data_aggregator_tx.clone();
        async move {
            services::market_manager::run_ws_exchanges(
                data_aggregator_tx,
            ).await;
        }
    });
    
    tokio::spawn({
        let client_aggregator_tx = client_aggregator_tx.clone();
        async move {
            transport::ws::connect_async(
                client_aggregator_tx,
            ).await;
        }
    });

    loop {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
