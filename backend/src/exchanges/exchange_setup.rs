use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use crate::{models::exchange::ExchangeType, services::exchange_store::{ExchangeStore, ExchangeStoreCMD}};

pub struct ExchangeSetup {
    pub title: String,
    pub enabled: bool,
    pub ticker_tx: async_channel::Sender<(String, String)>,
    pub ticker_rx: async_channel::Receiver<(String, String)>,
    pub channel_type: String,
    pub client: reqwest::Client,
    pub sender_data: mpsc::Sender<ExchangeStoreCMD>,
    pub books_updates: broadcast::Sender<String>,
}

impl ExchangeSetup {
    pub fn new(
        exchange_type: ExchangeType,
        enabled: bool
    ) -> Arc<Self> {
        let title = format!("{}{}Websocket ->", &exchange_type.to_string()[..1].to_uppercase(), &exchange_type.to_string()[1..]);
        let (ticker_tx, ticker_rx) = async_channel::bounded(1);
        let channel_type = String::from("spot");
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<ExchangeStoreCMD>(1);

        let store = ExchangeStore::new(
            rx_data, 
            exchange_type
        );
        let books_updates = store.book_updates.clone();
        tokio::spawn(async move {
            store.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled, channel_type,
            ticker_tx, ticker_rx, client,
            sender_data, books_updates,
        });

        this
    }
}