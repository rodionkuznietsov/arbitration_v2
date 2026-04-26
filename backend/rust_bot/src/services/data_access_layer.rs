use std::{collections::HashSet, sync::Arc, time::Duration};
use itertools::Itertools;
use tokio::sync::{Mutex, mpsc, oneshot, watch};
use crate::{models::exchange::ExchangeType, services::{cache_aggregator::CacheAggregatorCmd, data_aggregator::DataAggregatorCmd, data_mapping::DataMappingCmd, exchange::{exchange_aggregator::ExchangeStoreCMD, exchange_channel_store::ExchangeChannelStoreCmd}}};

/// Извлекает конкретные данные из:
/// 
/// • CacheAggregator(`LinesHistory`), 
/// <br>• ExchangeAggregator(`Quote`)
/// 
/// И после передаёт их в DataMapping
pub struct DataAccessLayer {
    cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
    data_mapping_tx: watch::Sender<DataMappingCmd>,
    exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>,
    data_aggregator_tx: watch::Sender<DataAggregatorCmd>,
    loaded_exchanges: Arc<Mutex<HashSet<ExchangeType>>>
}

impl DataAccessLayer {
    pub fn new(
        cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
        data_mapping_tx: watch::Sender<DataMappingCmd>,
        exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>,
        data_aggregator_tx: watch::Sender<DataAggregatorCmd>
    ) -> Arc<Self> {
        Arc::new(
            Self { 
                cache_aggregator_tx,
                data_mapping_tx,
                exchange_channel_store_tx,
                data_aggregator_tx,
                loaded_exchanges: Arc::new(Mutex::new(HashSet::new()))
            }
        )
    }

    pub async fn run(self: Arc<Self>) {
        tokio::spawn(self.clone().from_exchange_aggregator());
        tokio::spawn(self.clone().lines_from_cache_aggregator());
    }

    async fn lines_from_cache_aggregator(self: Arc<Self>) {        
        let (tx, mut rx) = mpsc::channel(1);
        
        self.cache_aggregator_tx.send(Arc::new(
            CacheAggregatorCmd::Subscribe { reply: tx }
        )).await.ok();
        
        if let Some(mut watch_rx) = rx.recv().await {
            while let Ok(_) = watch_rx.changed().await {
                let data = watch_rx.borrow().clone();
                let _ = self.data_mapping_tx.send(
                    DataMappingCmd::LinesFromDataAccessLayer(data), 
                );
            }
        }
    }

    /// Этот метод обращаюся к exchange_aggregator_tx за данными
    async fn from_exchange_aggregator(self: Arc<Self>) {
        let (tx, rx) = oneshot::channel();

        if let Some(err) = self.exchange_channel_store_tx.send_timeout(
            ExchangeChannelStoreCmd::GetExchangesChannel { 
                reply: tx
            },
            Duration::from_millis(5)
        ).await.err() {
            tracing::error!("{}", err);
        }
        
        if let Ok(mut watch_rx) = rx.await {
            
            while watch_rx.changed().await.is_ok() {
                let data = watch_rx.borrow_and_update().clone();
                for (exchange_id, exchange_aggregator_tx) in data.into_iter() {
                    if self.clone().exists_exchange(exchange_id).await {
                        continue;
                    }
                    let mut loaded_exchanges = self.loaded_exchanges.lock().await;
                    loaded_exchanges.insert(exchange_id);

                    // Подписываемся на обновления в ExchangeAggregator
                    let data_aggregator_tx = self.data_aggregator_tx.clone();
                    let (tx, mut rx) = mpsc::channel(1);
                    exchange_aggregator_tx.send(ExchangeStoreCMD::Subscribe { reply: tx }).ok();

                    if let Some(mut watch_aggregator_tx) = rx.recv().await {
                        tokio::spawn(async move {
                            while watch_aggregator_tx.changed().await.is_ok() {
                                let exchange_data = watch_aggregator_tx.borrow_and_update().clone();
 
                                let _ = data_aggregator_tx.send(
                                    DataAggregatorCmd::UpdateData { 
                                        exchange_id, 
                                        data: exchange_data
                                    }, 
                                );
                            }
                        });
                    }
                }
            }
        }
    }

    async fn exists_exchange(
        self: Arc<Self>,
        exchange_id: ExchangeType
    ) -> bool {
        let loaded_exchanges = self.loaded_exchanges.lock().await;
        loaded_exchanges.contains(&exchange_id)
    }
}