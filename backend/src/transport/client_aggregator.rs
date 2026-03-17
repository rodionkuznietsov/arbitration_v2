use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};
use tokio::sync::{mpsc, watch};
use tracing::{error};
use crate::{models::{aggregator::{AggregatorPayload, ClientAggregatorUse, KeyMarketType, KeyPair}, line::{Line, MarketType}, websocket::{ChannelSubscription, ClientId}}, services::cache_aggregator::CacheAggregatorCmd};

pub enum ClientMpcsChannel {
    OrderBook(watch::Sender<Arc<AggregatorPayload>>),
    Lines(mpsc::Sender<(VecDeque<Line>, MarketType)>)
}

pub enum ClientAggregatorCmd {
    Register {
        client_id: ClientId,
        tx: watch::Sender<Arc<AggregatorPayload>>,
        lines_tx: mpsc::Sender<(VecDeque<Line>, MarketType)>,
    },
    Use(ClientAggregatorUse),
    Init
}

pub struct ClientAggregator {
    watch_cmd_rx: watch::Receiver<Arc<ClientAggregatorCmd>>,
    cache_aggregator_tx: mpsc::Sender<CacheAggregatorCmd>,
    client_cmd_rx: mpsc::Receiver<ClientAggregatorCmd>,

    clients: HashMap<ClientId, Vec<ClientMpcsChannel>>,
    subscriptions: HashMap<ClientId, HashSet<ChannelSubscription>>,
    sub_index: HashMap<ChannelSubscription, HashSet<ClientId>>,
}

impl ClientAggregator {
    pub fn new(
        watch_cmd_rx: watch::Receiver<Arc<ClientAggregatorCmd>>,
        client_cmd_rx: mpsc::Receiver<ClientAggregatorCmd>,
        cache_aggregator_tx: mpsc::Sender<CacheAggregatorCmd>,
    ) -> Self {
        Self {
            watch_cmd_rx,
            cache_aggregator_tx,
            client_cmd_rx,

            clients: HashMap::new(),
            subscriptions: HashMap::new(),
            sub_index: HashMap::new(),
        }
    }
    
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Ok(_) = self.watch_cmd_rx.changed() => {
                    let cmd = self.watch_cmd_rx.borrow().clone();
                    self.handle_cmd(cmd).await;
                }

                Some(client_cmd) = self.client_cmd_rx.recv() => {
                    self.handle_cmd(Arc::new(client_cmd)).await;
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
                    .or_insert_with(Vec::new);

                entry.push(ClientMpcsChannel::OrderBook(tx.clone()));
                entry.push(ClientMpcsChannel::Lines(lines_tx.clone()));
            },
            ClientAggregatorCmd::Init => {},
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

                        // Отправляем последние 100 линий клиенту
                        match client_channel_sub {
                            ChannelSubscription::Chart { 
                                long_exchange, 
                                short_exchange, 
                                ticker 
                            } => {
                                if let Some(txs) = self.clients.get(&client_id) {
                                    for tx in txs {
                                        match tx {
                                            ClientMpcsChannel::Lines(
                                                channel_tx
                                            ) => {
                                                self.cache_aggregator_tx.send(
                                                CacheAggregatorCmd::GetLinesHistory { 
                                                        key: KeyPair::new(
                                                            KeyMarketType::new(*long_exchange, *short_exchange, ticker.clone()), 
                                                            KeyMarketType::new(*short_exchange, *long_exchange, ticker.clone()), 
                                                        ), 
                                                        reply: channel_tx.clone()
                                                    }
                                                ).await.ok();
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    ClientAggregatorUse::Publish { 
                        key,
                        payload,
                    } => {
                        if let Some(clients_ids) = self.sub_index.get(&key) {                    
                            for client_id in clients_ids {
                                if let Some(channels) = self.clients.get(&*client_id) {
                                    for tx in channels {
                                        match tx {
                                            ClientMpcsChannel::OrderBook(
                                                channel_tx
                                            ) => {
                                                match channel_tx.send(payload.clone()) {
                                                    Ok(_) => {},
                                                    Err(e) => {
                                                        error!("ClientAggregator: {}", e)
                                                    } 
                                                }
                                            },
                                            _ => {}
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

                                match channel_sub {
                                    ChannelSubscription::Chart { 
                                        long_exchange, 
                                        short_exchange, 
                                        ticker 
                                    } => {
                                        // Удаляем подписку в cache
                                        self.cache_aggregator_tx.send(
                                            CacheAggregatorCmd::RemovePair { 
                                                key: KeyPair { 
                                                    long_market_type: KeyMarketType { 
                                                        long_exchange: long_exchange, 
                                                        short_exchange: short_exchange,
                                                        symbol: ticker.clone()
                                                    }, 
                                                    short_market_type: KeyMarketType { 
                                                        long_exchange: short_exchange, 
                                                        short_exchange: long_exchange, 
                                                        symbol: ticker.clone()
                                                    }, 
                                                } 
                                            }
                                        ).await.ok();
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}