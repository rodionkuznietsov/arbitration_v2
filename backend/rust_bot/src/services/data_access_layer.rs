use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use crate::{services::{cache_aggregator::CacheAggregatorCmd, data_aggregator::DataAggregatorCmd, data_mapping::DataMappingCmd, exchange::{exchange_aggregator::ExchangeStoreCMD, exchange_channel_store::ExchangeChannelStoreCmd}}};

/// Извлекает конкретные данные из:
/// 
/// • CacheAggregator(`LinesHistory`), 
/// <br>• ExchangeAggregator(`Quote`)
/// 
/// И после передаёт их в DataMapping
pub struct DataAccessLayer {
    cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
    data_mapping_tx: mpsc::Sender<DataMappingCmd>,
    exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>,
    data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>
}

impl DataAccessLayer {
    pub fn new(
        cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
        data_mapping_tx: mpsc::Sender<DataMappingCmd>,
        exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>,
        data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>
    ) -> Arc<Self> {
        Arc::new(
            Self { 
                cache_aggregator_tx,
                data_mapping_tx,
                exchange_channel_store_tx,
                data_aggregator_tx
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
                self.data_mapping_tx.send_timeout(
                    DataMappingCmd::LinesFromDataAccessLayer(data), 
                    Duration::from_millis(300)
                ).await.ok();
            }
        }
    }

    /// Этот метод обращаюся к exchange_aggregator_tx за данными
    async fn from_exchange_aggregator(self: Arc<Self>) {
        let (tx, rx) = oneshot::channel();

        self.exchange_channel_store_tx.send_timeout(
            ExchangeChannelStoreCmd::GetExchangesChannel { 
                reply: tx
            },
            Duration::from_millis(300)
        ).await.ok();
        
        if let Ok(mut watch_rx) = rx.await {
            let exchanges_count = 3;

            // Возможно проблема здесь

            if watch_rx.borrow().len() == exchanges_count {
                let data = watch_rx.borrow_and_update().clone();
            } else {
                while watch_rx.changed().await.is_ok() {
                    let data = watch_rx.borrow_and_update().clone();
                    
                    if data.len() == exchanges_count {
                        for (exchange_id, exchange_aggregator_tx) in data.into_iter() {
                            let data_aggregator_tx = self.data_aggregator_tx.clone();

                            tokio::spawn(async move {
                                let data_aggregator_tx = data_aggregator_tx.clone();
                                let (tx, mut rx) = mpsc::channel(1);
                                exchange_aggregator_tx.send(ExchangeStoreCMD::Subscribe { reply: tx }).ok();

                                if let Some(mut watch_aggregator_tx) = rx.recv().await {
                                    while watch_aggregator_tx.changed().await.is_ok() {
                                        let (symbol, exchange_data) = watch_aggregator_tx.borrow().clone();
                                        
                                        if let Some(e) = data_aggregator_tx.send_timeout(
                                            DataAggregatorCmd::UpdateData { 
                                                exchange_id, 
                                                symbol,
                                                data: exchange_data
                                            }, 
                                        Duration::from_millis(10)
                                        ).await.err() {
                                            tracing::error!("DataAccessLayer(FromExchangeAggregator) -> {e};")
                                        }
                                    }
                                }
                            });
                        }
                        
                        break;
                    }
                }
            }
        }
    }
}