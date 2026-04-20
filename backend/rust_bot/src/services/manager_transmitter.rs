use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, watch};
use crate::{models::{aggregator::{ClientAggregatorUse}, websocket::{ChannelSubscription, WsClientMessage}}, services::cache_aggregator::CacheAggregatorCmd, transport::client_aggregator::ClientAggregatorCmd};

const TIMEOUT_DELAY: u64 = 30;

#[derive(Debug, Clone)]
pub enum NotifyEvent {
    #[allow(unused)]
    Cache(CacheAggregatorCmd),
    PayloadJson(
        ChannelSubscription,
        WsClientMessage
    ),
}

#[derive(Debug, Clone)]
pub enum ManagerTransmitterCmd {
    Notify(NotifyEvent),
    Default
}

#[derive(Clone)]
/// <b>ManagerTransmitter</b> ожидает обработанные данные и затем просто их отсылает далее
pub struct ManagerTransmitter {
    client_aggregator_chart_tx: mpsc::Sender<Arc<ClientAggregatorCmd>>,
    cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
}

impl ManagerTransmitter {
    pub fn new(
        client_aggregator_chart_tx: mpsc::Sender<Arc<ClientAggregatorCmd>>,
        cache_aggregator_tx: mpsc::Sender<Arc<CacheAggregatorCmd>>,
    ) -> Self {
        Self { 
            client_aggregator_chart_tx,
            cache_aggregator_tx,
        }
    }

    pub async fn run(
        self, 
        mut notify_rx: watch::Receiver<ManagerTransmitterCmd>,
    ) {
        while notify_rx.changed().await.is_ok() {
            let cmd = notify_rx.borrow().clone();

            match cmd {
                ManagerTransmitterCmd::Notify(event) => {
                    match event {
                        NotifyEvent::Cache(cmd) => {
                            self.cache_aggregator_tx.send_timeout(
                                Arc::new(
                                    cmd.clone()
                                ),
                                Duration::from_millis(TIMEOUT_DELAY)
                            ).await.ok();
                        },
                        NotifyEvent::PayloadJson(
                            key,
                            msg
                        ) => {
                            if let Some(err) = self.client_aggregator_chart_tx.send_timeout(
                                Arc::new(
                                    ClientAggregatorCmd::Use(
                                        ClientAggregatorUse::PublishJson(
                                            key.clone(), 
                                            msg.clone()
                                        )
                                    )
                                ),
                                Duration::from_millis(TIMEOUT_DELAY)
                            ).await.err() {
                                tracing::error!("{}", err);
                            }
                        },
                    }
                },
                ManagerTransmitterCmd::Default => {}
            } 
        }
    }
}