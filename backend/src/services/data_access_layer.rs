use std::{sync::Arc, time::Duration};
use get_size::GetSize;
use tokio::sync::{mpsc, oneshot};
use crate::services::{cache_aggregator::CacheAggregatorCmd, data_aggregator::{DataAggregatorCmd}, data_mapping::DataMappingCmd, exchange::{exchange_aggregator::ExchangeStoreCMD, exchange_channel_store::ExchangeChannelStoreCmd}};

const LINES_DELAY: u64 = 100;

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
        loop {
            let (reply_tx, mut reply_rx) = mpsc::channel(1);

            if let Some(err) = self.cache_aggregator_tx.clone().send_timeout(
                Arc::new(
                    CacheAggregatorCmd::LinesHistory { 
                        reply: reply_tx.clone()
                    }
                ),
                Duration::from_millis(LINES_DELAY)
            ).await.err() {
                tracing::error!("DataAccessLayer(LinesFromCacheAggregator) -> {err}")
            };

            let mut buffer = Vec::new();
            let _ = reply_rx.recv_many(&mut buffer, 100).await;
            
            for msg in buffer.drain(..) {
                if msg.len() > 0 {
                    let bytes = msg.get_size();
                    println!("{:?}", bytes);
                    // if let Some(e) = self.data_mapping_tx.send_timeout(
                    //     DataMappingCmd::LinesToJsonPair(
                    //         msg
                    //     ),
                    //     Duration::from_millis(LINES_DELAY)
                    // ).await.err() {
                    //     tracing::error!("DataAccessLayer(LinesFromCacheAggregator) -> {e}; Capacity: {}", self.data_mapping_tx.capacity())
                    // };
                }
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
            Duration::from_millis(100)
        ).await.ok();
        
        if let Ok(mut watch_rx) = rx.await {
            let exchanges_count = 3;

            if watch_rx.borrow().len() == exchanges_count {
                let data = watch_rx.borrow_and_update().clone();
                println!("{data:?}")
            } else {
                while watch_rx.changed().await.is_ok() {
                    let data = watch_rx.borrow_and_update().clone();
                    
                    if data.len() == exchanges_count {
                        for (exchange_id, exchange_aggregator_tx) in data.into_iter() {
                            let data_aggregator_tx = self.data_aggregator_tx.clone();

                            tokio::spawn(async move {
                                let data_aggregator_tx = data_aggregator_tx.clone();
                                let (tx, rx) = oneshot::channel();
                                exchange_aggregator_tx.send(ExchangeStoreCMD::Subscribe { reply: tx }).await.ok();

                                if let Ok(mut watch_aggregator_tx) = rx.await {
                                    let _new_data = watch_aggregator_tx.borrow().clone();

                                    while watch_aggregator_tx.changed().await.is_ok() {
                                        let (symbol, exchange_data) = watch_aggregator_tx.borrow().clone();
                                        if let Some(e) = data_aggregator_tx.send_timeout(
                                            DataAggregatorCmd::UpdateData { 
                                                exchange_id, 
                                                symbol,
                                                data: exchange_data
                                            }, 
                                        Duration::from_millis(100)
                                        ).await.err() {
                                            tracing::error!("DataAccessLayer(FromExchangeAggregator) -> {e}; capacity: {}", data_aggregator_tx.capacity())
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