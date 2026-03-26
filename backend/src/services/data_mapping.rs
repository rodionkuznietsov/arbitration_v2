use std::{collections::{HashMap, VecDeque}, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{models::{aggregator::{JsonPairData, KeyMarketType}, exchange::ExchangeType, exchange_aggregator::BookData, line::Line, orderbook::Snapshot, websocket::{ChannelSubscription, ChannelType, Symbol, WsClientMessage, WsClientMsgResult}}, services::manager_transmitter::{ManagerTransmitterCmd, NotifyEvent}};

pub enum DataMappingCmd {
    #[allow(unused)]
    /// Этот команда приводит `lines` в формат JSON и соединяет попарно (`Long Lines & Short Lines`)
    LinesToJsonPair(HashMap<KeyMarketType, VecDeque<Line>>),
    /// Этот команда приводит `ExchangesData` в формат JSON и соединяет попарно (`Long data & Short data`)
    ExchangesDataToJsonPair(Vec<(ExchangeType, Arc<Symbol>, Arc<BookData>)>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SnapshotJson {
    asks: Vec<Value>,
    bids: Vec<Value>,
    last_price: f64,
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
        let (data_mapping_tx, data_mapping_rx) = mpsc::channel::<DataMappingCmd>(100);
        Self { 
            data_mapping_tx,
            data_mapping_rx,
            manager_transmitter_tx
        }
    }

    pub fn run(mut self) {
        tokio::spawn(async move {
            loop {
                if let Some(cmd) = self.data_mapping_rx.recv().await {
                    match cmd {
                        DataMappingCmd::LinesToJsonPair(
                            lines
                        ) => {
                            for (i, (long_key, long_lines)) in lines.iter().enumerate() {
                                for (short_key, short_lines) in lines.iter().skip(i+1) {
                                    
                                    let long_lines_json = self.lines_to_json(long_lines);
                                    let short_lines_json = self.lines_to_json(short_lines);

                                    let msg = WsClientMessage {
                                        channel: ChannelType::Chart,
                                        result: WsClientMsgResult {
                                            data: JsonPairData::LinesHistory { 
                                                long: long_lines_json, 
                                                short: short_lines_json 
                                            }.into(),
                                            symbol: long_key.symbol.clone()
                                        }
                                    }; 

                                    let channel_key = ChannelSubscription::Chart { 
                                        long_market_type: KeyMarketType { 
                                            long_exchange: long_key.long_exchange, 
                                            short_exchange: long_key.short_exchange, 
                                            symbol: long_key.symbol.clone()
                                        }, 
                                        short_market_type: KeyMarketType { 
                                            long_exchange: short_key.long_exchange, 
                                            short_exchange: short_key.short_exchange, 
                                            symbol: short_key.symbol.clone()
                                        }, 
                                    };

                                    self.manager_transmitter_tx.send_timeout(
                                        ManagerTransmitterCmd::Notify(
                                            NotifyEvent::PayloadJson(
                                                channel_key,
                                                msg
                                            )
                                        ), 
                                    Duration::from_millis(100)
                                    ).await.ok();
                                }
                            }
                        },
                        DataMappingCmd::ExchangesDataToJsonPair(
                            markets
                        ) => {
                            for (i, (long_ex_id, symbol, long_data)) in markets.iter().enumerate() {
                                for (short_ex_id, _, short_data) in markets.iter().skip(i+1) {
                                    let long_json_lines = self.snapshot_to_json(&long_data.snapshot, &long_data.last_price);
                                    let short_json_lines = self.snapshot_to_json(&short_data.snapshot, &short_data.last_price);
                                    
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
                                                        long, 
                                                        short 
                                                    }
                                                ), 
                                                symbol: symbol.clone()
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
                                        Duration::from_millis(10)
                                        ).await.err() {
                                            tracing::error!("DataMapping -> {err}");
                                        }
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
        map: &Option<Snapshot>,
        last_price: &Option<f64>
    ) -> Option<SnapshotJson> {
        if let (Some(snapshot), Some(last_price)) = (map, last_price) {
            let snapshot_ui = snapshot.to_ui(6, *last_price);
            let asks_json: Vec<Value> = self.ask_bid_to_json(snapshot_ui.a);
            let bids_json: Vec<Value> = self.ask_bid_to_json(snapshot_ui.b);
            
            let snapshot = SnapshotJson {
                asks: asks_json,
                bids: bids_json,
                last_price: *last_price
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
        map
            .into_iter()
            .map(|line| {
                serde_json::json!({
                    "time": line.timestamp,
                    "value": line.value
                })
            }).collect()
    }
}
