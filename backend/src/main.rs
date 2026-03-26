use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc};
use tracing_subscriber::EnvFilter;

use crate::{services::{cache_aggregator::{CacheAggregator, CacheAggregatorCmd}, data_access_layer::DataAccessLayer, data_aggregator::{DataAggregator, DataAggregatorCmd}, data_mapping::DataMapping, exchange::exchange_channel_store::ExchangeChannelStore, manager_transmitter::{ManagerTransmitter, ManagerTransmitterCmd}}, transport::client_aggregator::{ClientAggregator, ClientAggregatorCmd}};

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
        .with_env_filter(
            EnvFilter::new("info,sqlx::query=off")
        )
        .with_target(false)
        .pretty()
        .init();
    
    let storage_pool = storage::pool::create_pool().await;
        
    let (manager_transmitter_tx, manager_transmitter_rx) = mpsc::channel::<ManagerTransmitterCmd>(32);
        
    let data_mapping = DataMapping::new(manager_transmitter_tx.clone());
    let data_mapping_tx = data_mapping.data_mapping_tx.clone();
    data_mapping.run();

    // Запускаем агррегаторы
    let (cache_aggregator_tx, cache_aggregator_rx) = mpsc::channel::<Arc<CacheAggregatorCmd>>(1);
    let cache_aggregator = CacheAggregator::new(
        cache_aggregator_rx, 
        storage_pool.clone()
    );
    tokio::spawn(cache_aggregator.run());

    // Каналы для получения данных с data aggregator
    let (client_aggregator_chart_tx, client_aggregator_chart_rx) = mpsc::channel::<Arc<ClientAggregatorCmd>>(128);

    // Канал для приёма команд от пользователя
    let (client_aggregator_tx, client_aggregator_rx) = mpsc::channel::<ClientAggregatorCmd>(32);

    let client_aggregator = ClientAggregator::new(
        client_aggregator_rx,
        client_aggregator_chart_rx
    );
    tokio::spawn(client_aggregator.run());
    
    let (data_aggregator_tx, data_aggregator_rx) = mpsc::channel::<DataAggregatorCmd>(32);
    let data_aggregator = DataAggregator::new(
        data_aggregator_rx, 
        data_mapping_tx.clone(),
        storage_pool.clone(),
    );

    let manager_transmitter = ManagerTransmitter::new(
        client_aggregator_chart_tx.clone(),
        cache_aggregator_tx.clone(),
    );
    tokio::spawn(manager_transmitter.run(manager_transmitter_rx));

    let exchange_channel_store = ExchangeChannelStore::new();
    let exchange_channel_store_tx = exchange_channel_store.sender_channel.clone();
    tokio::spawn(exchange_channel_store.run());

    let data_access_layer = DataAccessLayer::new(
        cache_aggregator_tx.clone(),
        data_mapping_tx.clone(),
        exchange_channel_store_tx.clone(),
        data_aggregator_tx.clone()
    );
    tokio::spawn(data_access_layer.run());

    tokio::spawn(
        data_aggregator.run(
            data_mapping_tx.clone(),
        )
    );

    // Запуск биржевых вебсокетов
    tokio::spawn({
        let data_aggregator_tx = data_aggregator_tx.clone();
        async move {
            services::exchange::exchanges_run::run_ws_exchanges(
                data_aggregator_tx,
                exchange_channel_store_tx
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
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
