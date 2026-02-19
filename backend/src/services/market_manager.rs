use std::{str::FromStr, sync::Arc};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{exchange::{ExchangeType, SharedSpreads}, line::{Line, TimeFrame}, orderbook::{OrderType, SnapshotUi}, websocket::{ChannelType, ServerToClientEvent}}, services::spread::{calculate_spread_for_chart, spawn_local_spread_engine}, storage::candle::{add_new_line, get_user_candles}, transport::ws::ConnectedClient};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)>;
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>);
    async fn get_spread(self: Arc<Self>, spread_tx: mpsc::Sender<Option<(ExchangeType, Option<f64>, Option<f64>)>>);
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
    pool: sqlx::PgPool
) {

    let kucoin_websocket = KuCoinWebsocket::new(true);
    let bybit_websocket = BybitWebsocket::new(true);
    let binx_websocket = BinXWebsocket::new(false);
    let mexc_websocket = MexcWebsocket::new(false);
    let binance_websocket = BinanceWebsocket::new(false);
    let gate_websocket = GateWebsocket::new(true);
    let lbank_websocket = LBankWebsocket::new(false);

    let shared_spreads = Arc::new(SharedSpreads::new());

    spawn_local_spread_engine(
        bybit_websocket.clone(),
        gate_websocket.clone(),
        kucoin_websocket.clone(),
        shared_spreads.clone(),
    );

    let (spread_tx, _) = broadcast::channel::<(String, OrderedFloat<f64>)>(100);
    let sender_for_calc = Arc::new(spread_tx.clone());
    calculate_spread_for_chart(
        shared_spreads, sender_for_calc
    );

    let (line_tx, mut line_rx) = mpsc::channel::<Line>(1);

    tokio::spawn({
        let pool = pool.clone();
        async move {
            while let Some(line) = line_rx.recv().await {
                match add_new_line(&pool, line).await {
                    Ok(_) => { println!("Новая точка добавлена") },
                    Err(e) => { eprintln!("[MarketManager]: {e}") }
                }
            }
        }
    });

    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let exchange_pair = client.exchange_pair.clone();
        let ticker = client.ticker.clone();
        let client = client.clone();
        let spread_tx = spread_tx.clone();
        let line_tx = line_tx.clone();

        // Инициализация последних 100 свечей
        tokio::spawn({
            let pool = pool.clone();
            let mut client = client.clone();
            let token = token.clone();
            let ticker = ticker.clone();
            
            async move {
                if !exchange_pair.is_empty() {
                    let init_lines = get_user_candles(&pool, &ticker, &exchange_pair).await;
                    let last_line = Arc::new(Mutex::new(Line::new()));

                    if let Ok(lines) = init_lines {
                        if lines.len() > 0 {
                            let index_last_element = lines.len()-1;
                            if let Some(line) = lines.get(index_last_element) {
                                let mut lock = last_line.lock().await;
                                *lock = line.clone();
                            }
                            
                            tokio::select! {
                                _ = token.cancelled() => return,
                                _ = client.send_to_client(
                                        ServerToClientEvent::LinesHistory(
                                        ChannelType::LinesHistory, 
                                        lines,
                                        ticker.clone()
                                    )
                                ) => {}
                            }
                        }
                    }

                    let mut spread_rx = spread_tx.subscribe();
                    let ticker = ticker.clone();
                    let line_tx = line_tx.clone();
                    
                    let last_line_cl = last_line.lock().await.clone();
                    let last_time = last_line_cl.timestamp;
                    let last_timestamp = last_time.timestamp();
                    let mut last_minute = last_timestamp - (last_timestamp % 60);

                    tokio::spawn(async move {
                        loop {
                            tokio::select! {
                                _ = token.cancelled() => break,
                                result = spread_rx.recv() => {
                                    match result {
                                        Ok((pair, spread)) => {
                                            if pair != exchange_pair {
                                                continue;
                                            }
                                            
                                            let now = Utc::now();
                                            let ts = now.timestamp();
                                            let minute_start = ts - (ts % 60);

                                            // println!("Start: {}; End: {}", minute_start, last_minute);
                                            
                                            if minute_start == last_minute {
                                                client.send_to_client(
                                                    ServerToClientEvent::UpdateLine(
                                                        ChannelType::LinesHistory,
                                                        Line {
                                                            timestamp: Utc.timestamp_opt(last_minute, 0).unwrap(),
                                                            exchange_pair: pair,
                                                            symbol: ticker.clone(),
                                                            timeframe: TimeFrame::Five,
                                                            value: BigDecimal::from_str(&format!("{}", spread)).unwrap()
                                                        }, 
                                                        ticker.clone()
                                                    )
                                                ).await;
                                            } else if minute_start > last_timestamp {
                                                println!("Добавление данных");
                                                last_minute = minute_start;

                                                let new_line = Line {
                                                    timestamp: Utc.timestamp_opt(last_minute, 0).unwrap(),
                                                    exchange_pair: pair,
                                                    symbol: ticker.clone(),
                                                    timeframe: TimeFrame::Five,
                                                    value: BigDecimal::from_str(&format!("{}", spread)).unwrap()
                                                };

                                                if line_tx.clone().send(new_line.clone()).await.is_err() {
                                                    continue
                                                }

                                                client.send_to_client(
                                                    ServerToClientEvent::UpdateHistory(
                                                        ChannelType::UpdateHistory, new_line.clone()
                                                    )
                                                ).await;
                                            }
                                        },
                                        Err(_) => break 
                                    }
                                }
                            };
                        }
                    });
                }
                return;
            }
        });

        tokio::spawn({
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();
            let binance = binance_websocket.clone();
            let gate = gate_websocket.clone();
            let lbank = lbank_websocket.clone();

            let token = token.clone();
            let client = client.clone();

            async move {
                if long_exchange != ExchangeType::Unknown {
                    let websocket: Arc<dyn ExchangeWebsocket> = match long_exchange {
                        ExchangeType::Binance => binance.clone(),
                        ExchangeType::Bybit => bybit.clone(),
                        ExchangeType::KuCoin => kucoin.clone(),
                        ExchangeType::BinX => binx.clone(),
                        ExchangeType::Mexc => mexc.clone(),
                        ExchangeType::Gate => gate.clone(),
                        ExchangeType::LBank => lbank.clone(),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::channel(100);
                    
                    tokio::spawn({
                        let token  = token.clone();
                        async move {
                            tokio::select! {
                                _ = token.cancelled() => return,
                                _ = websocket.get_snapshot(snapshot_tx) => {}
                            }
                        }
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      
                        let ticker = client.ticker.clone();

                        tokio::select! {
                            _ = token.cancelled() => return,
                            _ = client.send_to_client(
                                    ServerToClientEvent::OrderBook(
                                    ChannelType::OrderBook, 
                                    OrderType::Long, 
                                    snapshot, 
                                    ticker
                                )
                            ) => {}
                        }
                    }

                    return;
                }
            }
        });

        tokio::spawn({
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();
            let binance = binance_websocket.clone();
            let gate = gate_websocket.clone();
            let lbank = lbank_websocket.clone();
            
            let token = token.clone();
            let client = client.clone();

            async move {
                if short_exchange != ExchangeType::Unknown {
                    let websocket: Arc<dyn ExchangeWebsocket> = match short_exchange {
                        ExchangeType::Binance => binance.clone(),
                        ExchangeType::Bybit => bybit.clone(),
                        ExchangeType::KuCoin => kucoin.clone(),
                        ExchangeType::BinX => binx.clone(),
                        ExchangeType::Mexc => mexc.clone(),
                        ExchangeType::Gate => gate.clone(),
                        ExchangeType::LBank => lbank.clone(),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::channel(100);
                    
                    tokio::spawn({
                        let token  = token.clone();
                        async move {
                            tokio::select! {
                                _ = token.cancelled() => return,
                                _ = websocket.get_snapshot(snapshot_tx) => {}
                            }
                        }
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      
                        let ticker = client.ticker.clone();

                        tokio::select! {
                            _ = token.cancelled() => return ,
                            _ = client.send_to_client(
                                    ServerToClientEvent::OrderBook(
                                    ChannelType::OrderBook, 
                                    OrderType::Short, 
                                    snapshot, 
                                    ticker
                                )
                            ) => {}
                        }
                    }

                    return ;
                }
            }
        });
    }
}