use std::{collections::{HashMap, VecDeque}, sync::Arc, time::Duration};

use tokio::sync::mpsc;

use crate::{models::{aggregator::{AggregatorPayload, ClientAggregatorUse, KeyMarketType}, line::Line, websocket::{ChannelSubscription, ChartEvent}}, transport::client_aggregator::ClientAggregatorCmd};

#[derive(Debug)]
pub enum NotifyEvent {
    LinesHistory(HashMap<KeyMarketType, VecDeque<Line>>),
    Payload(
        ChannelSubscription,
        Arc<AggregatorPayload>,
        ChannelSubscription,
        Arc<AggregatorPayload>
    ),
}

#[derive(Debug)]
pub enum ManagerTransmitterCmd {
    Notify(NotifyEvent)
}

/// Ждёт новые данные и не вникая в суть передаёт их дальше
pub struct ManagerTransmitter {
    client_aggregator_chart_tx: mpsc::Sender<Arc<ClientAggregatorCmd>>
}

impl ManagerTransmitter {
    pub fn new(
        client_aggregator_chart_tx: mpsc::Sender<Arc<ClientAggregatorCmd>>
    ) -> Self {
        Self { 
            client_aggregator_chart_tx
        }
    }

    pub async fn run(
        self, 
        mut notify_rx: mpsc::Receiver<ManagerTransmitterCmd>
    ) {
        loop {
            if let Some(cmd) = notify_rx.recv().await {
                match cmd {
                    ManagerTransmitterCmd::Notify(event) => {
                        match event {
                            NotifyEvent::LinesHistory(cache_lines) => {
                                for (key, lines) in cache_lines {
                                    let k = ChannelSubscription::Chart { 
                                        long_exchange: key.long_exchange, 
                                        short_exchange: key.short_exchange, 
                                        ticker: key.symbol.clone()
                                    };

                                    let payload = Arc::new(
                                    AggregatorPayload::ChartEvent { 
                                            event: ChartEvent::LinesHistory(lines), 
                                            ticker: key.symbol
                                        }
                                    );
                                    
                                    self.client_aggregator_chart_tx.send_timeout(
                                        Arc::new(
                                            ClientAggregatorCmd::Use(
                                                ClientAggregatorUse::Publish { key: k, payload: payload }
                                            )
                                        ), 
                                        Duration::from_millis(30)
                                    ).await.ok();
                                }
                            },
                            NotifyEvent::Payload(
                                long_key, 
                                long_payload,
                                short_key, 
                                short_payload,
                            ) => {
                                self.client_aggregator_chart_tx.send_timeout(
                                    Arc::new(ClientAggregatorCmd::Use(
                                            ClientAggregatorUse::Publish { 
                                                key: long_key,
                                                payload: long_payload
                                            }
                                        )),
                                        Duration::from_millis(30)
                                ).await.ok();

                                self.client_aggregator_chart_tx.send_timeout(
                                    Arc::new(ClientAggregatorCmd::Use(
                                            ClientAggregatorUse::Publish { 
                                                key: short_key,
                                                payload: short_payload
                                            }
                                        )),
                                        Duration::from_millis(30)
                                ).await.ok();
                            }
                        }
                    }
                }
            }
        }
    }
}