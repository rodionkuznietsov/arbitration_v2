use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::warn;
use crate::{models::exchange::ExchangeType, services::{aggregator::AggregatorCommand, exchange_store::{ExchangeStore, ExchangeStoreCMD}, market_manager::ExchangeWebsocket}};

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
    exchange_id: ExchangeType
}

impl ExchangeSetup {
    pub fn new(
        exchange_id: ExchangeType,
        enabled: bool,
    ) -> Arc<Self> {
        let title = format!("{}{}Websocket ->", &exchange_id.to_string()[..1].to_uppercase(), &exchange_id.to_string()[1..]);
        let (ticker_tx, ticker_rx) = async_channel::bounded(1);
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<ExchangeStoreCMD>(1);

        let store = ExchangeStore::new(
            rx_data, 
            exchange_id
        );
        let books_updates = store.book_updates.clone();
        tokio::spawn(async move {
            store.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, channel_type,
            ticker_tx, ticker_rx, client,
            sender_data, books_updates,
            exchange_id
        });

        this
    }
}

#[async_trait]
impl ExchangeWebsocket for ExchangeSetup {
    fn spawn_quote_updater(
        self: Arc<Self>,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    ) {
        let mut rx = self.books_updates.subscribe();
        let title = self.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ticker) => {
                        let (reply, rx) = oneshot::channel::<(f64, f64)>();
                        if self.sender_data.send(ExchangeStoreCMD::Quote { ticker: ticker.clone(), reply: reply }).await.is_err() {
                            continue;
                        }

                        if let Ok((ask, bid)) = rx.await {
                            if aggregator_tx.clone().send(
                                AggregatorCommand::UpdateQuotes { 
                                    exchange_type: self.exchange_id,
                                    ticker: ticker.clone(),
                                    ask,
                                    bid
                                }
                            ).await.is_err() {
                                continue;
                            }
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
}