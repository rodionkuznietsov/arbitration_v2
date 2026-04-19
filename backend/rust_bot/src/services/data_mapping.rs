use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc, time::Duration};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};

use crate::{models::{aggregator::{JsonPairData, JsonPairUniqueId, KeyMarketType, SpreadPair, Volume}, exchange::ExchangeType, line::{Line}, orderbook::Snapshot, websocket::{ChannelSubscription, ChannelType, Symbol, WsClientMessage, WsClientMsgResult}}, services::manager_transmitter::{ManagerTransmitterCmd, NotifyEvent}};

const MANAGER_TRANSMITTER_TIMEOUT_DELAY: u64 = 300; // ms

pub enum DataMappingCmd {
    #[allow(unused)]
    /// Это команда приводит `lines` в формат JSON и соединяет попарно (`Long Lines & Short Lines`)
    LinesFromDataAccessLayer(Arc<RwLock<Arc<HashMap<(ExchangeType, ExchangeType), HashMap<Arc<Symbol>, Arc<RwLock<VecDeque<Line>>>>>>>>),
    LinesToJsonPair(Arc<RwLock<VecDeque<Line>>>, Arc<RwLock<VecDeque<Line>>>, Arc<Symbol>, ExchangeType, ExchangeType),
    LinesFromDbToJsonPair(HashMap<(ExchangeType, ExchangeType, Arc<Symbol>), VecDeque<Line>>),
    /// Это команда приводит `ExchangesData` в формат JSON и соединяет попарно (`Long data & Short data`)
    ExchangesDataToJsonPair(Vec<(ExchangeType, Arc<Symbol>, (Option<Arc<Snapshot>>, Option<f64>))>),
    /// Это команда приводит `SpreadPair` в формат JSON и соединяет попарно (`Long Spread & Short Spread`)
    SpreadPairToJsonPair(Arc<SpreadPair>),
    /// Это команда приводит `Volumes` в формат JSON и соединяет попарно (`Long Volume & Short Volume`)
    VolumesToJson(Vec<Volume>)
}

#[derive(Debug, Deserialize, Clone, Serialize, Hash, PartialEq, Eq)]
pub struct SnapshotJson {
    pub asks: Vec<Value>,
    pub bids: Vec<Value>,
    pub last_price: OrderedFloat<f64>,
}

pub struct DataMapping {
    pub data_mapping_tx: mpsc::Sender<DataMappingCmd>,
    data_mapping_rx: mpsc::Receiver<DataMappingCmd>,
    manager_transmitter_tx: mpsc::Sender<ManagerTransmitterCmd>
}

impl DataMapping {
    pub fn new(
        manager_transmitter_tx: mpsc::Sender<ManagerTransmitterCmd>
    ) -> Self {
        let (data_mapping_tx, data_mapping_rx) = mpsc::channel::<DataMappingCmd>(1024);
        Self { 
            data_mapping_tx,
            data_mapping_rx,
            manager_transmitter_tx
        }
    }

    pub fn run(mut self) {
        tokio::spawn(async move {
            loop {
                let mut data = Vec::new();
                let _ = self.data_mapping_rx.recv_many(&mut data, 100).await;
                
                for cmd in data.drain(..) {
                    match cmd {
                        DataMappingCmd::LinesFromDataAccessLayer(
                            data
                        ) => {
                            let lines = data.read().await;
                            for (i, ((long_exchange, _), data)) in lines.iter().enumerate() {
                                for ((short_long_exchange, _), short_data) in lines.iter().skip(i+1) {
                                    for (symbol, long_vec) in data {
                                        for (short_symbol, short_vec) in short_data {
                                            let long_vec = long_vec.read().await;
                                            let short_vec = short_vec.read().await;

                                            if symbol == short_symbol {
                                                let long_json_lines = self.lines_to_json(&long_vec);
                                                let short_json_lines = self.lines_to_json(&short_vec);

                                                let msg = WsClientMessage {
                                                    channel: ChannelType::Chart,
                                                    result: WsClientMsgResult {
                                                        data: JsonPairData::LinesHistory { 
                                                            long: long_json_lines.clone(), 
                                                            short: short_json_lines.clone()
                                                        }.into(),
                                                        symbol: symbol.clone(),
                                                        unique_id: JsonPairUniqueId::LinesHistory
                                                    }
                                                }; 

                                                let channel_key = ChannelSubscription::Chart { 
                                                    long_market_type: KeyMarketType { 
                                                        long_exchange: *long_exchange, 
                                                        short_exchange: *short_long_exchange, 
                                                        symbol: symbol.clone()
                                                    }, 
                                                    short_market_type: KeyMarketType { 
                                                        long_exchange: *short_long_exchange, 
                                                        short_exchange: *long_exchange, 
                                                        symbol: symbol.clone()
                                                    }, 
                                                };

                                                if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                                    ManagerTransmitterCmd::Notify(
                                                        NotifyEvent::PayloadJson(
                                                            channel_key,
                                                            msg
                                                        )
                                                    ), 
                                                Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                                ).await.err() {
                                                    tracing::error!("{{ data_mapping.lines_from_data_access_layer.first }} -> {err}");
                                                }

                                                let msg_2 = WsClientMessage {
                                                    channel: ChannelType::Chart,
                                                    result: WsClientMsgResult {
                                                        data: JsonPairData::LinesHistory { 
                                                            long: short_json_lines.clone(), 
                                                            short: long_json_lines.clone()
                                                        }.into(),
                                                        symbol: symbol.clone(),
                                                        unique_id: JsonPairUniqueId::LinesHistory
                                                    }
                                                }; 

                                                let channel_key_2 = ChannelSubscription::Chart { 
                                                    long_market_type: KeyMarketType { 
                                                        long_exchange: *short_long_exchange, 
                                                        short_exchange: *long_exchange, 
                                                        symbol: symbol.clone()
                                                    }, 
                                                    short_market_type: KeyMarketType { 
                                                        long_exchange: *long_exchange, 
                                                        short_exchange: *short_long_exchange, 
                                                        symbol: symbol.clone()
                                                    }, 
                                                };

                                                if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                                    ManagerTransmitterCmd::Notify(
                                                        NotifyEvent::PayloadJson(
                                                            channel_key_2,
                                                            msg_2
                                                        )
                                                    ), 
                                                Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                                ).await.err() {
                                                    tracing::error!("{{ data_mapping.lines_from_data_access_layer.last }} -> {err}");
                                                }                                              
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        DataMappingCmd::LinesToJsonPair(
                            long_data,
                            short_data,
                            symbol,
                            long_exchange, 
                            short_exchange
                        ) => {
                            let long_lines = long_data.read().await;
                            let short_lines = short_data.read().await;

                            let long_json_lines = self.lines_to_json(&long_lines);
                            let short_json_lines = self.lines_to_json(&short_lines);

                            let msg = WsClientMessage {
                                channel: ChannelType::Chart,
                                result: WsClientMsgResult {
                                    data: JsonPairData::LinesHistory { 
                                        long: long_json_lines.clone(), 
                                        short: short_json_lines.clone()
                                    }.into(),
                                    symbol: symbol.clone(),
                                    unique_id: JsonPairUniqueId::LinesHistory
                                }
                            }; 

                            let channel_key = ChannelSubscription::Chart { 
                                long_market_type: KeyMarketType { 
                                    long_exchange: long_exchange, 
                                    short_exchange: short_exchange, 
                                    symbol: symbol.clone()
                                }, 
                                short_market_type: KeyMarketType { 
                                    long_exchange: short_exchange, 
                                    short_exchange: long_exchange, 
                                    symbol: symbol.clone()
                                }, 
                            };

                            if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                ManagerTransmitterCmd::Notify(
                                    NotifyEvent::PayloadJson(
                                        channel_key,
                                        msg
                                    )
                                ), 
                            Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                            ).await.err() {
                                tracing::error!("{{ data_mapping.lines_to_json_pair.first }} -> {err}");
                            }

                            let msg_2 = WsClientMessage {
                                channel: ChannelType::Chart,
                                result: WsClientMsgResult {
                                    data: JsonPairData::LinesHistory { 
                                        long: short_json_lines, 
                                        short: long_json_lines,
                                    }.into(),
                                    symbol: symbol.clone(),
                                    unique_id: JsonPairUniqueId::LinesHistory
                                }
                            }; 

                            let channel_key_2 = ChannelSubscription::Chart { 
                                long_market_type: KeyMarketType { 
                                    long_exchange: short_exchange, 
                                    short_exchange: long_exchange, 
                                    symbol: symbol.clone()
                                }, 
                                short_market_type: KeyMarketType { 
                                    long_exchange: long_exchange, 
                                    short_exchange: short_exchange, 
                                    symbol: symbol.clone()
                                }, 
                            };

                            if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                ManagerTransmitterCmd::Notify(
                                    NotifyEvent::PayloadJson(
                                        channel_key_2,
                                        msg_2
                                    )
                                ), 
                            Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                            ).await.err() {
                                tracing::error!("{{ data_mapping.lines_to_json_pair.last }} -> {err}");
                            }
                        },
                        DataMappingCmd::LinesFromDbToJsonPair(lines) => {
                            for (i, ((long_exchange, short_exchnage, symbol), long_lines)) in lines.iter().enumerate() {
                                for (_, short_lines) in lines.iter().skip(i+1) {
                                    let long_lines = self.lines_to_json(&long_lines);
                                    let short_lines = self.lines_to_json(&short_lines);
                                    
                                    let msg = WsClientMessage {
                                        channel: ChannelType::Chart,
                                        result: WsClientMsgResult { 
                                            data: JsonPairData::LinesHistory { 
                                                long: long_lines.clone(), 
                                                short: short_lines.clone() 
                                            }.into(), 
                                            symbol: symbol.clone(), 
                                            unique_id: JsonPairUniqueId::LinesHistory
                                        },
                                    };

                                    let channel_key = ChannelSubscription::Chart { 
                                        long_market_type: KeyMarketType { 
                                            long_exchange: *long_exchange, 
                                            short_exchange: *short_exchnage, 
                                            symbol: symbol.clone() 
                                        }, 
                                        short_market_type: KeyMarketType { 
                                            long_exchange: *short_exchnage, 
                                            short_exchange: *long_exchange, 
                                            symbol: symbol.clone() 
                                        }, 
                                    };

                                    if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                        ManagerTransmitterCmd::Notify(
                                            NotifyEvent::PayloadJson(
                                                channel_key,
                                                msg
                                            )
                                        ), 
                                    Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                    ).await.err() {
                                        tracing::error!("{{ data_mapping.lines_from_db_to_json_pair.first }} -> {err}");
                                    }

                                    let msg_2 = WsClientMessage {
                                        channel: ChannelType::Chart,
                                        result: WsClientMsgResult { 
                                            data: JsonPairData::LinesHistory { 
                                                long: short_lines, 
                                                short: long_lines 
                                            }.into(), 
                                            symbol: symbol.clone(), 
                                            unique_id: JsonPairUniqueId::LinesHistory
                                        },
                                    };

                                    let channel_key_2 = ChannelSubscription::Chart { 
                                        long_market_type: KeyMarketType { 
                                            long_exchange: *short_exchnage, 
                                            short_exchange: *long_exchange, 
                                            symbol: symbol.clone() 
                                        }, 
                                        short_market_type: KeyMarketType { 
                                            long_exchange: *long_exchange, 
                                            short_exchange: *short_exchnage, 
                                            symbol: symbol.clone() 
                                        }, 
                                    };

                                    if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                        ManagerTransmitterCmd::Notify(
                                            NotifyEvent::PayloadJson(
                                                channel_key_2,
                                                msg_2
                                            )
                                        ), 
                                    Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                    ).await.err() {
                                        tracing::error!("{{ data_mapping.lines_from_db_to_json_pair.last }} -> {err}");
                                    }

                                }
                            }
                        },
                        DataMappingCmd::ExchangesDataToJsonPair(
                            markets
                        ) => {

                            for (i, (long_ex_id, symbol, (long_snapshot, long_last_price))) in markets.iter().enumerate() {
                                for (short_ex_id, _, (short_snapshot, short_last_price)) in markets.iter().skip(i+1) {
                                    tracing::info!("{}/{} -> {}; {:?} {:?}", long_ex_id, short_ex_id, symbol, short_last_price, long_last_price);
                                    
                                    let long_json_lines = self.snapshot_to_json(long_snapshot, &long_last_price);
                                    let short_json_lines = self.snapshot_to_json(short_snapshot, &short_last_price);
                                    
                                    if let (
                                        Some(long), 
                                        Some(short)
                                    ) = (
                                        long_json_lines, 
                                        short_json_lines
                                    ) {
                                        let msg = WsClientMessage {
                                            channel: ChannelType::OrderBook,
                                            result: WsClientMsgResult { 
                                                data: Arc::new(
                                                    JsonPairData::OrderBook { 
                                                        long: long.clone(), 
                                                        short: short.clone(),
                                                    }
                                                ), 
                                                symbol: symbol.clone(),
                                                unique_id: JsonPairUniqueId::OrderBook
                                            },
                                        };

                                        let channel_key = ChannelSubscription::OrderBook { 
                                            long_market_type: KeyMarketType { 
                                                long_exchange: *long_ex_id, 
                                                short_exchange: *short_ex_id, 
                                                symbol: symbol.clone()
                                            }, 
                                            short_market_type: KeyMarketType { 
                                                long_exchange: *short_ex_id, 
                                                short_exchange: *long_ex_id, 
                                                symbol: symbol.clone()
                                            }, 
                                        };

                                        if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                            ManagerTransmitterCmd::Notify(
                                                NotifyEvent::PayloadJson(
                                                    channel_key, 
                                                    msg
                                                )
                                            ), 
                                        Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                        ).await.err() {
                                            tracing::error!("{{ data_mapping.exchanges_data_to_json_pair.first }} -> {err}");
                                        }

                                        let msg_2 = WsClientMessage {
                                            channel: ChannelType::OrderBook,
                                            result: WsClientMsgResult { 
                                                data: Arc::new(
                                                    JsonPairData::OrderBook { 
                                                        long: short, 
                                                        short: long,
                                                    }
                                                ), 
                                                symbol: symbol.clone(),
                                                unique_id: JsonPairUniqueId::OrderBook
                                            },
                                        };

                                        let channel_key_2 = ChannelSubscription::OrderBook { 
                                            long_market_type: KeyMarketType { 
                                                long_exchange: *short_ex_id, 
                                                short_exchange: *long_ex_id, 
                                                symbol: symbol.clone()
                                            }, 
                                            short_market_type: KeyMarketType { 
                                                long_exchange: *long_ex_id, 
                                                short_exchange: *short_ex_id, 
                                                symbol: symbol.clone()
                                            }, 
                                        };

                                        if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                            ManagerTransmitterCmd::Notify(
                                                NotifyEvent::PayloadJson(
                                                    channel_key_2, 
                                                    msg_2
                                                )
                                            ), 
                                        Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                        ).await.err() {
                                            tracing::error!("{{ data_mapping.exchanges_data_to_json_pair.last }} -> {err}");
                                        }
                                    }
                                }
                            }
                        },
                        DataMappingCmd::SpreadPairToJsonPair(
                            spread_pair
                        ) => {
                            let long = serde_json::json!({
                                "value": spread_pair.long_spread,
                                "time": spread_pair.timestamp.clone()
                            });

                            let short = serde_json::json!({
                                "value": spread_pair.short_spread,
                                "time": spread_pair.timestamp.clone()
                            });
                            
                            let msg = WsClientMessage {
                                channel: ChannelType::Chart,
                                result: WsClientMsgResult { 
                                    data: Arc::new(
                                        JsonPairData::UpdateLine { 
                                            long: long.clone(), 
                                            short: short.clone(),
                                        }
                                    ), 
                                    symbol: spread_pair.symbol.clone(),
                                    unique_id: JsonPairUniqueId::UpdateLine
                                },
                            };

                            let channel_key = ChannelSubscription::OrderBook { 
                                long_market_type: KeyMarketType { 
                                    long_exchange: spread_pair.long_exchange, 
                                    short_exchange: spread_pair.short_exchange, 
                                    symbol: spread_pair.symbol.clone()
                                }, 
                                short_market_type: KeyMarketType { 
                                    long_exchange: spread_pair.short_exchange, 
                                    short_exchange: spread_pair.long_exchange, 
                                    symbol: spread_pair.symbol.clone()
                                }, 
                            };

                            if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                ManagerTransmitterCmd::Notify(
                                    NotifyEvent::PayloadJson(
                                        channel_key, 
                                        msg
                                    )
                                ), 
                            Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                            ).await.err() {
                                tracing::error!("{{ data_mapping.spread_pair_to_json_pair.first }} -> {err}");
                            }
                            
                            let msg_2 = WsClientMessage {
                                channel: ChannelType::Chart,
                                result: WsClientMsgResult { 
                                    data: Arc::new(
                                        JsonPairData::UpdateLine { 
                                            long: short, 
                                            short: long,
                                        }
                                    ), 
                                    symbol: spread_pair.symbol.clone(),
                                    unique_id: JsonPairUniqueId::UpdateLine
                                },
                            };

                            let channel_key_2 = ChannelSubscription::OrderBook { 
                                long_market_type: KeyMarketType { 
                                    long_exchange: spread_pair.short_exchange, 
                                    short_exchange: spread_pair.long_exchange, 
                                    symbol: spread_pair.symbol.clone()
                                }, 
                                short_market_type: KeyMarketType { 
                                    long_exchange: spread_pair.long_exchange, 
                                    short_exchange: spread_pair.short_exchange, 
                                    symbol: spread_pair.symbol.clone()
                                }, 
                            };

                            if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                ManagerTransmitterCmd::Notify(
                                    NotifyEvent::PayloadJson(
                                        channel_key_2, 
                                        msg_2
                                    )
                                ), 
                            Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                            ).await.err() {
                                tracing::error!("{{ data_mapping.spread_pair_to_json_pair.last }} -> {err}");
                            }
                        }, 
                        DataMappingCmd::VolumesToJson(
                            volumes
                        ) => {
                            for (i, long_vol) in volumes.iter().enumerate() {
                                for short_vol in volumes.iter().skip(i+1) {
                                    let msg = WsClientMessage {
                                        channel: ChannelType::Chart,
                                        result: WsClientMsgResult { 
                                            data: JsonPairData::Volume24h { 
                                                long: serde_json::to_value(long_vol).unwrap(), 
                                                short: serde_json::to_value(short_vol).unwrap()
                                            }.into(), 
                                            symbol: long_vol.symbol.clone(), 
                                            unique_id: JsonPairUniqueId::Volume24h
                                        },
                                    };

                                    let channel_key = ChannelSubscription::Chart { 
                                        long_market_type: KeyMarketType { 
                                            long_exchange: long_vol.exchange_id, 
                                            short_exchange: short_vol.exchange_id, 
                                            symbol: long_vol.symbol.clone(),
                                        }, 
                                        short_market_type: KeyMarketType { 
                                            long_exchange: short_vol.exchange_id, 
                                            short_exchange: long_vol.exchange_id, 
                                            symbol: short_vol.symbol.clone(),
                                        },
                                    };

                                    if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                        ManagerTransmitterCmd::Notify(
                                            NotifyEvent::PayloadJson(
                                                channel_key, 
                                                msg
                                            )
                                        ), 
                                    Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                    ).await.err() {
                                        tracing::error!("{{ data_mapping.volumes_to_json.first }} -> {err}");
                                    }

                                    let msg_2 = WsClientMessage {
                                        channel: ChannelType::Chart,
                                        result: WsClientMsgResult { 
                                            data: JsonPairData::Volume24h { 
                                                long: serde_json::to_value(short_vol).unwrap(), 
                                                short: serde_json::to_value(long_vol).unwrap()
                                            }.into(), 
                                            symbol: short_vol.symbol.clone(), 
                                            unique_id: JsonPairUniqueId::Volume24h
                                        },
                                    };

                                    let channel_key_2 = ChannelSubscription::Chart { 
                                        long_market_type: KeyMarketType { 
                                            long_exchange: short_vol.exchange_id, 
                                            short_exchange: long_vol.exchange_id, 
                                            symbol: short_vol.symbol.clone(),
                                        }, 
                                        short_market_type: KeyMarketType { 
                                            long_exchange: long_vol.exchange_id, 
                                            short_exchange: short_vol.exchange_id, 
                                            symbol: long_vol.symbol.clone(),
                                        },
                                    };

                                    if let Some(err) = self.manager_transmitter_tx.send_timeout(
                                        ManagerTransmitterCmd::Notify(
                                            NotifyEvent::PayloadJson(
                                                channel_key_2, 
                                                msg_2
                                            )
                                        ), 
                                    Duration::from_millis(MANAGER_TRANSMITTER_TIMEOUT_DELAY)
                                    ).await.err() {
                                        tracing::error!("{{ data_mapping.volumes_to_json.last }} -> {err}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn snapshot_to_json(
        &self,
        map: &Option<Arc<Snapshot>>,
        last_price: &Option<f64>
    ) -> Option<SnapshotJson> {
        if let (Some(snapshot), Some(last_price)) = (map, last_price) {
            let snapshot_ui = snapshot.to_ui(6, *last_price);
            let asks_json: Vec<Value> = self.ask_bid_to_json(snapshot_ui.a);
            let bids_json: Vec<Value> = self.ask_bid_to_json(snapshot_ui.b);
            
            let snapshot = SnapshotJson {
                asks: asks_json,
                bids: bids_json,
                last_price: OrderedFloat(*last_price)
            };

            return Some(snapshot);
        } else {
            None
        }
    }

    fn ask_bid_to_json(
        &self,
        map: Vec<(f64, f64)>
    ) -> Vec<Value> {
        map
            .into_iter()
            .map(|(price, volume)| {
                serde_json::json!({
                    "price": price,
                    "volume": volume
                })
            }).collect()
    }

    fn lines_to_json(
        &self,
        map: &VecDeque<Line>
    ) -> Vec<Value> {
        let mut seen = HashSet::new();
        
        map
            .into_iter()
            .filter(|line| seen.insert(line.timestamp))
            .sorted_by_key(|l| l.timestamp)
            .map(|line| {
                serde_json::json!({
                    "time": line.timestamp,
                    "value": line.value
                })
            }).collect()
    }
}
