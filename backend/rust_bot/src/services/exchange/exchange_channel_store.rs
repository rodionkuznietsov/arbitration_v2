use std::{collections::HashMap};
use tokio::sync::{mpsc, oneshot, watch};
use crate::{models::exchange::ExchangeType, services::exchange::exchange_aggregator::ExchangeStoreCMD};

pub enum ExchangeChannelStoreCmd {
    RegisterChannel {
        exchange_id: ExchangeType,
        channel: watch::Sender<ExchangeStoreCMD>
    },
    
    GetExchangesChannel {
        reply: oneshot::Sender<watch::Receiver<HashMap<ExchangeType, watch::Sender<ExchangeStoreCMD>>>>
    },

    Default
}

pub struct ExchangeChannelStore {
    exchanges_channel: HashMap<ExchangeType, watch::Sender<ExchangeStoreCMD>>,

    pub sender_channel: mpsc::Sender<ExchangeChannelStoreCmd>,
    receiver_channel: mpsc::Receiver<ExchangeChannelStoreCmd>,
    watch: watch::Sender<HashMap<ExchangeType, watch::Sender<ExchangeStoreCMD>>>,
    watch_rx: watch::Receiver<HashMap<ExchangeType, watch::Sender<ExchangeStoreCMD>>>
}

impl ExchangeChannelStore {
    pub fn new() -> Self {
        let (sender_channel, receiver_channel) = mpsc::channel(32);
        let (watch, watch_rx) = watch::channel(HashMap::new());
        
        Self { 
            exchanges_channel: HashMap::new(),

            sender_channel, receiver_channel,

            watch, watch_rx
        }
    }

    pub async fn run(mut self) {
        loop {
            if let Some(cmd) = self.receiver_channel.recv().await {
                match cmd {
                    ExchangeChannelStoreCmd::RegisterChannel { 
                        exchange_id, 
                        channel 
                    } => {
                        self.exchanges_channel.insert(exchange_id, channel);
                        self.watch.send(self.exchanges_channel.clone()).ok();
                    },
                    ExchangeChannelStoreCmd::GetExchangesChannel { 
                        reply 
                    } => {
                        let _ = reply.send(self.watch_rx.clone());
                    },
                    ExchangeChannelStoreCmd::Default => {}
                }
            }
        }
    }
}