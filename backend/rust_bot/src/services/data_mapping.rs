use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc};
use futures_util::future::join_all;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{RwLock, mpsc, watch};

use crate::{models::{aggregator::{JsonPairData, JsonPairUniqueId, KeyMarketType, SpreadPair, Volume}, data_mapping::{DataJson, SnapshotJson}, exchange::ExchangeType, line::Line, orderbook::Snapshot, websocket::{ChannelSubscription, ChannelType, Symbol, WsClientMessage, WsClientMsgResult}}, services::manager_transmitter::{ManagerTransmitterCmd, NotifyEvent}};

const MANAGER_TRANSMITTER_TIMEOUT_DELAY: u64 = 10; // ms

#[derive(Clone)]
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
    VolumesToJson(Vec<Volume>),
    Default
}

pub struct DataMapping {
    pub data_mapping_tx: watch::Sender<DataMappingCmd>,
    data_mapping_rx: watch::Receiver<DataMappingCmd>,
    manager_transmitter_tx: mpsc::Sender<ManagerTransmitterCmd>
}

impl DataMapping {
    pub fn new(
        manager_transmitter_tx: mpsc::Sender<ManagerTransmitterCmd>
    ) -> Self {
        let (data_mapping_tx, data_mapping_rx) = watch::channel::<DataMappingCmd>(DataMappingCmd::Default);
        Self { 
            data_mapping_tx,
            data_mapping_rx,
            manager_transmitter_tx
        }
    }

    pub fn run(mut self) {
        tokio::spawn(async move {
            while let Ok(_) = self.data_mapping_rx.changed().await {
                let cmd = self.data_mapping_rx.borrow().clone();
                
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
                                            let long_json_lines = Arc::new(self.lines_to_json(&long_vec));
                                            let short_json_lines = Arc::new(self.lines_to_json(&short_vec));

                                            // self.send_message_with_key(
                                            //     ChannelType::Chart,
                                            //     *long_exchange,
                                            //     DataJson::LinesHistory(long_json_lines.clone()),
                                            //     *short_long_exchange,
                                            //     DataJson::LinesHistory(short_json_lines.clone()),
                                            //     symbol.clone(),
                                            //     JsonPairUniqueId::LinesHistory
                                            // ).await;

                                            // self.send_message_with_key(
                                            //     ChannelType::Chart,
                                            //     *short_long_exchange,
                                            //     DataJson::LinesHistory(short_json_lines),
                                            //     *long_exchange,
                                            //     DataJson::LinesHistory(long_json_lines),
                                            //     symbol.clone(),
                                            //     JsonPairUniqueId::LinesHistory
                                            // ).await;
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

                        let long_json_lines = Arc::new(self.lines_to_json(&long_lines));
                        let short_json_lines = Arc::new(self.lines_to_json(&short_lines));

                        // self.send_message_with_key(
                        //     ChannelType::Chart,
                        //     long_exchange,
                        //     DataJson::LinesHistory(long_json_lines.clone()),
                        //     short_exchange,
                        //     DataJson::LinesHistory(short_json_lines.clone()),
                        //     symbol.clone(),
                        //     JsonPairUniqueId::LinesHistory
                        // ).await;

                        // self.send_message_with_key(
                        //     ChannelType::Chart,
                        //     short_exchange,
                        //     DataJson::LinesHistory(short_json_lines),
                        //     long_exchange,
                        //     DataJson::LinesHistory(long_json_lines),
                        //     symbol.clone(),
                        //     JsonPairUniqueId::LinesHistory
                        // ).await;
                    },
                    DataMappingCmd::LinesFromDbToJsonPair(lines) => {
                        for (i, ((long_exchange, short_exchnage, symbol), long_lines)) in lines.iter().enumerate() {
                            for (_, short_lines) in lines.iter().skip(i+1) {
                                let long_lines = Arc::new(self.lines_to_json(&long_lines));
                                let short_lines = Arc::new(self.lines_to_json(&short_lines));
                                
                                // self.send_message_with_key(
                                //     ChannelType::Chart,
                                //     *long_exchange,
                                //     DataJson::LinesHistory(long_lines.clone()),
                                //     *short_exchnage,
                                //     DataJson::LinesHistory(short_lines.clone()),
                                //     symbol.clone(),
                                //     JsonPairUniqueId::LinesHistory
                                // ).await;

                                // self.send_message_with_key(
                                //     ChannelType::Chart,
                                //     *short_exchnage,
                                //     DataJson::LinesHistory(short_lines),
                                //     *long_exchange,
                                //     DataJson::LinesHistory(long_lines),
                                //     symbol.clone(),
                                //     JsonPairUniqueId::LinesHistory
                                // ).await;
                            }
                        }
                    },
                    DataMappingCmd::ExchangesDataToJsonPair(
                        markets
                    ) => {
                        let mut futures = Vec::new();
                        for (i, (long_ex_id, symbol, (long_snapshot, long_last_price))) in markets.iter().enumerate() {
                            for (short_ex_id, short_symbol, (short_snapshot, short_last_price)) in markets.iter().skip(i+1) {
                                let long_json_lines = self.snapshot_to_json(long_snapshot, &long_last_price);
                                let short_json_lines = self.snapshot_to_json(short_snapshot, &short_last_price);

                                if let (
                                    Some(long), 
                                    Some(short)
                                ) = (
                                    long_json_lines, 
                                    short_json_lines
                                ) { 
                                    let long_arc = Arc::new(long);
                                    let short_arc = Arc::new(short);
                                    
                                    // self.send_message_with_key(
                                    //     ChannelType::OrderBook,
                                    //     *long_ex_id,
                                    //     JsonPairData::OrderBook { long: long_arc.clone(), short: short_arc.clone() },
                                    //     *short_ex_id,
                                    //     symbol.clone(),
                                    //     JsonPairUniqueId::OrderBook,
                                    // ).await;

                                    // self.send_message_with_key(
                                    //     ChannelType::OrderBook,
                                    //     *short_ex_id,
                                    //     JsonPairData::OrderBook { long: short_arc.clone(), short: long_arc.clone() },
                                    //     *long_ex_id,
                                    //     short_symbol.clone(),
                                    //     JsonPairUniqueId::OrderBook,
                                    // ).await;

                                    futures.push(self.send_message_with_key(
                                        ChannelType::OrderBook,
                                        *long_ex_id,
                                        JsonPairData::OrderBook { long: long_arc.clone(), short: short_arc.clone() },
                                        *short_ex_id,
                                        symbol.clone(),
                                        JsonPairUniqueId::OrderBook,
                                    ));

                                    futures.push(self.send_message_with_key(
                                        ChannelType::OrderBook,
                                        *short_ex_id,
                                        JsonPairData::OrderBook { long: short_arc.clone(), short: long_arc.clone() },
                                        *long_ex_id,
                                        short_symbol.clone(),
                                        JsonPairUniqueId::OrderBook,
                                    ));
                                }
                            }
                        }

                        let _ = join_all(futures).await;
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

                        // self.send_message_with_key(
                        //     ChannelType::Chart,
                        //     spread_pair.long_exchange,
                        //     DataJson::UpdateLine(long),
                        //     spread_pair.short_exchange,
                        //     DataJson::UpdateLine(short),
                        //     spread_pair.symbol.clone(),
                        //     JsonPairUniqueId::UpdateLine
                        // ).await;
                    }, 
                    DataMappingCmd::VolumesToJson(
                        volumes
                    ) => {
                        for (i, long_vol) in volumes.iter().enumerate() {
                            for short_vol in volumes.iter().skip(i+1) {
                                // self.send_message_with_key(
                                //     ChannelType::Chart,
                                //     *&long_vol.exchange_id,
                                //     DataJson::Volume24h(serde_json::to_value(long_vol).unwrap()),
                                //     *&short_vol.exchange_id,
                                //     DataJson::Volume24h(serde_json::to_value(short_vol).unwrap()),
                                //     long_vol.symbol.clone(),
                                //     JsonPairUniqueId::Volume24h
                                // ).await;

                                // self.send_message_with_key(
                                //     ChannelType::Chart,
                                //     *&short_vol.exchange_id,
                                //     DataJson::Volume24h(serde_json::to_value(short_vol).unwrap()),
                                //     *&long_vol.exchange_id,
                                //     DataJson::Volume24h(serde_json::to_value(long_vol).unwrap()),
                                //     short_vol.symbol.clone(),
                                //     JsonPairUniqueId::Volume24h
                                // ).await;
                            }
                        }
                    }, 
                    DataMappingCmd::Default => {}
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

    async fn send_message_with_key(
        &self,
        channel: ChannelType,
        long_exchange: ExchangeType,
        data: JsonPairData,
        short_exchange: ExchangeType,
        symbol: Arc<Symbol>,
        unique_id: JsonPairUniqueId,
    ) {
        let msg = WsClientMessage {
            channel: channel,
            result: WsClientMsgResult { 
                data: Arc::new(data), 
                symbol: symbol.clone(),
                unique_id: unique_id
            },
        };

        let channel_key = ChannelSubscription::OrderBook { 
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

        let _ = self.manager_transmitter_tx.send(
            ManagerTransmitterCmd::Notify(
                NotifyEvent::PayloadJson(
                    channel_key.clone(), 
                    msg.clone()
                )
            ), 
        ).await;
    }
}
