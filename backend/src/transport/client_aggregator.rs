use std::{collections::{HashMap, HashSet}, sync::Arc};
use tokio::sync::mpsc;
use crate::models::{aggregator::{AggregatorPayload, ClientAggregatorCmd}, websocket::{ChannelSubscription, ClientId}};

pub struct ClientAggregator {
    pub cmd_rx: mpsc::Receiver<ClientAggregatorCmd>,
    pub clients: HashMap<ClientId, mpsc::Sender<Arc<AggregatorPayload>>>,
    pub subscriptions: HashMap<ClientId, HashSet<ChannelSubscription>>,
    pub sub_index: HashMap<ChannelSubscription, HashSet<ClientId>>,
}

impl ClientAggregator {
    pub fn new(cmd_rx: mpsc::Receiver<ClientAggregatorCmd>) -> Self {
        Self {
            cmd_rx,
            clients: HashMap::new(),
            subscriptions: HashMap::new(),
            sub_index: HashMap::new(),
        }
    }
    
    pub async fn run(mut self) {
        loop {
            if let Some(cmd) = self.cmd_rx.recv().await {
                self.handle_cmd(cmd).await;
            }
        }
    }

    pub async fn handle_cmd(
        &mut self, 
        cmd: ClientAggregatorCmd
    ) {
        match cmd {
            ClientAggregatorCmd::Subscribe(
                client_id, 
                client_channel_sub,
            ) => {
                self.subscriptions
                    .entry(client_id)
                    .or_insert_with(HashSet::new)
                    .insert(client_channel_sub.clone());

                self.sub_index.entry(client_channel_sub)
                    .or_insert_with(HashSet::new)
                    .insert(client_id);
            },
            ClientAggregatorCmd::Publish { 
                key,
                payload 
            } => {
                if let Some(clients_ids) = self.sub_index.get(&key) {                    
                    for client_id in clients_ids {
                        if let Some(client_tx) = self.clients.get(&*client_id) {
                            client_tx.send(payload.clone()).await.ok();
                        }
                    }
                }
            },
            ClientAggregatorCmd::Register { id, tx } => {
                self.clients.entry(id).or_insert_with(|| tx);
            },
            ClientAggregatorCmd::UnRegister(client_id) => {
                self.clients.remove(&client_id);
                if let Some(subs) = self.subscriptions.remove(&client_id) {
                    for sub in subs {
                        self.sub_index.remove(&sub);
                    }
                }
                println!("{:?}", self.subscriptions);
                println!("{:?}", self.sub_index);
            }
        }
    }
}