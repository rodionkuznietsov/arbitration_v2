use std::{collections::{HashMap, HashSet}, sync::Arc, time::Duration};
use tokio::sync::{mpsc};
use crate::{models::{aggregator::{ClientAggregatorUse}, websocket::{ChannelSubscription, ChannelType, ClientId, WsClientMessage}}};

#[derive(Debug)]
pub enum ClientMpcsChannel {
    OrderBook(mpsc::Sender<Arc<WsClientMessage>>),
    #[allow(unused)]
    Lines(mpsc::Sender<Arc<WsClientMessage>>)
}

pub enum ClientAggregatorCmd {
    Register {
        client_id: ClientId,
        tx: mpsc::Sender<Arc<WsClientMessage>>,
        lines_tx: mpsc::Sender<Arc<WsClientMessage>>,
    },
    Use(ClientAggregatorUse),
}

pub struct ClientAggregator {
    client_cmd_rx: mpsc::Receiver<ClientAggregatorCmd>,
    cmd_rx: mpsc::Receiver<Arc<ClientAggregatorCmd>>,

    clients: HashMap<ClientId, HashMap<ChannelType, ClientMpcsChannel>>,
    subscriptions: HashMap<ClientId, HashSet<ChannelSubscription>>,
    sub_index: HashMap<ChannelSubscription, HashSet<ClientId>>,
}

impl ClientAggregator {
    pub fn new(
        client_cmd_rx: mpsc::Receiver<ClientAggregatorCmd>,
        cmd_rx: mpsc::Receiver<Arc<ClientAggregatorCmd>>,
    ) -> Self {
        Self {
            client_cmd_rx,
            cmd_rx,

            clients: HashMap::new(),
            subscriptions: HashMap::new(),
            sub_index: HashMap::new(),
        }
    }
    
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                biased;
                Some(client_cmd) = self.client_cmd_rx.recv() => {
                    self.handle_cmd(Arc::new(client_cmd)).await;
                }

                Some(client_cmd) = self.cmd_rx.recv() => {
                    self.handle_cmd(client_cmd).await;
                }
            }
        }
    }

    async fn handle_cmd(
        &mut self, 
        cmd: Arc<ClientAggregatorCmd>
    ) {
        match cmd.as_ref() {
            ClientAggregatorCmd::Register { 
                client_id, 
                tx ,
                lines_tx,
            } => {
                let entry = self.clients
                    .entry(*client_id)
                    .or_insert_with(HashMap::new);

                entry.insert(ChannelType::OrderBook, ClientMpcsChannel::OrderBook(tx.clone()));
                entry.insert(ChannelType::Chart, ClientMpcsChannel::Lines(lines_tx.clone()));
            },
            ClientAggregatorCmd::Use (
                use_cmd 
            ) => {
                match use_cmd {
                    ClientAggregatorUse::Subscribe(
                        client_id, 
                        client_channel_sub,
                    ) => {
                        self.subscriptions
                            .entry(*client_id)
                            .or_insert_with(HashSet::new)
                            .insert(client_channel_sub.clone());

                        self.sub_index.entry(client_channel_sub.clone())
                            .or_insert_with(HashSet::new)
                            .insert(*client_id);

                        println!("Subscriptions: {:?}", self.subscriptions);
                        println!("\nSub Index: {:?}", self.sub_index);
                    },
                    ClientAggregatorUse::PublishJson(
                        key,
                        msg,
                    ) => {
                        if let Some(client_ids) = self.sub_index.get(&key) {
                            for client_id in client_ids {
                                if let Some(channels) = self.clients.get(&*client_id) {
                                    if let Some(ch) = channels.get(&msg.channel) {
                                        match ch {
                                            ClientMpcsChannel::OrderBook(channel_tx) => {
                                                channel_tx.send_timeout(
                                                    Arc::new(msg.clone()), 
                                                    Duration::from_millis(10)
                                                ).await.ok();
                                            },
                                            ClientMpcsChannel::Lines(channel_tx) => {
                                                channel_tx.send_timeout(
                                                    Arc::new(msg.clone()), 
                                                    Duration::from_millis(10)
                                                ).await.ok();
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    },
                    ClientAggregatorUse::UnRegister(
                        client_id
                    ) => {
                        self.clients.remove(&client_id);
                        if let Some(subs) = self.subscriptions.remove(&client_id) {
                            for sub in subs {
                                if let Some(clients) = self.sub_index.get_mut(&sub) {
                                    clients.remove(&*client_id);
                                }
                            }
                        }

                        // Удаляем channel_sub из sub_index, если нет клиентов
                        for (channel_sub, clients) in self.sub_index.clone() {
                            if clients.is_empty() {
                                self.sub_index.remove(&channel_sub);
                            }
                        }
                    }
                }
            }
        }
    }
}