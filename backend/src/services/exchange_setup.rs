use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::warn;
use crate::models::{exchange::ExchangeType, orderbook::SnapshotUi};
use crate::services::{aggregator::AggregatorCommand, exchange_aggregator::{ExchangeStore, ExchangeStoreCMD}};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn spawn_quote_updater(self: Arc<Self>);
    fn spawn_volume_updater(self: Arc<Self>);
    fn spawn_oderbooks_updater(self: Arc<Self>);
}

/// Init Exchange
pub struct ExchangeSetup {
    pub title: String,
    pub enabled: bool,
    #[allow(unused)]
    pub ticker_tx: async_channel::Sender<(String, String)>,
    #[allow(unused)]
    pub ticker_rx: async_channel::Receiver<(String, String)>,
    pub channel_type: String,
    pub client: reqwest::Client,
    pub sender_data: mpsc::Sender<ExchangeStoreCMD>,
    pub books_updates: broadcast::Sender<String>,
    exchange_id: ExchangeType,

    aggregator_tx: mpsc::Sender<AggregatorCommand>
}

impl ExchangeSetup {
    pub fn new(
        exchange_id: ExchangeType,
        enabled: bool,
        aggregator_tx: mpsc::Sender<AggregatorCommand> 
    ) -> Arc<Self> {
        let title = format!("{}{}Websocket ->", &exchange_id.to_string()[..1].to_uppercase(), &exchange_id.to_string()[1..]);
        let (ticker_tx, ticker_rx) = async_channel::bounded(1);
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<ExchangeStoreCMD>(1);

        let store = ExchangeStore::new(rx_data);
        let books_updates = store.book_updates.clone();
        tokio::spawn(async move {
            store.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, channel_type,
            ticker_tx, ticker_rx, client,
            sender_data, books_updates,
            exchange_id, aggregator_tx
        });

        this.clone().spawn_quote_updater();
        this.clone().spawn_volume_updater();
        this.clone().spawn_oderbooks_updater();

        this
    }
}

#[async_trait]
impl ExchangeWebsocket for ExchangeSetup {
    fn spawn_quote_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        let title = self.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        let (reply, rx) = oneshot::channel::<(f64, f64)>();
                        if self.sender_data.send(ExchangeStoreCMD::GetQuote { ticker: ticker.clone(), reply: reply }).await.is_err() {
                            continue;
                        }

                        if let Ok((ask, bid)) = rx.await {
                            self.aggregator_tx.clone().send(
                                AggregatorCommand::UpdateQuotes { 
                                    exchange_type: self.exchange_id,
                                    ticker: ticker.clone(),
                                    ask,
                                    bid
                                }
                            ).await.ok();
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал спреда закрыт", title);
                        break;
                    }
                }
            }
        });
    }

    fn spawn_volume_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        let title = self.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        let (reply, rx) = oneshot::channel::<f64>();
                        if self.sender_data.send(ExchangeStoreCMD::GetVolume { ticker: ticker.clone(), reply }).await.is_err() {
                            continue;
                        }

                        if let Ok(volume) = rx.await {
                            self.aggregator_tx.clone().send(
                                AggregatorCommand::UpdateVolumes { 
                                    exchange_type: self.exchange_id, 
                                    volume, 
                                    ticker
                                }
                            ).await.ok();
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал спреда закрыт", title);
                        break;
                    }
                }
            }
        });
    }

    fn spawn_oderbooks_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        let (reply, rx) = oneshot::channel::<SnapshotUi>();
                        self.sender_data.send(ExchangeStoreCMD::GetBook { ticker: ticker.clone(), reply }).await.ok();

                        if let Ok(snapshot) = rx.await {
                            self.aggregator_tx.send(AggregatorCommand::UpdateOrderbooks { 
                                exchange_type: self.exchange_id,
                                snapshot_ui: snapshot,
                                ticker,
                            }).await.ok();
                        }   
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    _ => continue
                }
            }
        });
    }
}
