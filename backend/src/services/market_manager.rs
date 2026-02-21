use std::{str::FromStr, sync::Arc};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use ordered_float::OrderedFloat;
use sqlx::types::BigDecimal;
use tokio::sync::{broadcast, mpsc};

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{exchange::{ExchangeType, SharedSpreads}, line::{Line, TimeFrame}, orderbook::{MarketType, SnapshotUi}, websocket::{ChannelType, ServerToClientEvent}}, services::spread::{calculate_spread_for_chart, spawn_local_spread_engine, spawn_spread_db_wirter}, storage::line_storage::{add_new_line, get_spread_history}, transport::ws::ConnectedClient};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)>;
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>);
    async fn get_spread(self: Arc<Self>, spread_tx: mpsc::Sender<Option<(ExchangeType, String, Option<f64>, Option<f64>)>>);
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

    let (spread_tx, _) = broadcast::channel::<(String, String, OrderedFloat<f64>)>(100);
    let sender_for_calc = Arc::new(spread_tx.clone());
    calculate_spread_for_chart(
        shared_spreads, sender_for_calc
    );

    let (new_line_tx, mut new_line_rx) = mpsc::channel::<Line>(1);
    spawn_spread_db_wirter(
        TimeFrame::One,
        spread_tx.clone(),
        new_line_tx.clone(),
        pool.clone()
    );

    tokio::spawn({
        let pool = pool.clone();
        async move {
            while let Some(line) = new_line_rx.recv().await {
                match add_new_line(&pool, line.clone()).await {
                    Ok(_) => { println!("Новая точка добавлена для пары: {}", line.exchange_pair) },
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

        // Инициализация последних 100 свечей
        tokio::spawn({
            let client = client.clone();
            let token = token.clone();
            let ticker = ticker.clone();
            let pool = pool.clone();
            let spread_tx = spread_tx.clone();

            async move {
                if !exchange_pair.long_pair.is_empty() {
                    let init_long_lines = get_spread_history(&pool, &ticker, &exchange_pair.long_pair).await;
                    tokio::spawn({
                        let ticker = ticker.clone();
                        let token = token.clone();
                        let mut client = client.clone();
                        let mut spread_rx = spread_tx.subscribe();

                        async move {

                            // let last_timestamp =  

                            if let Ok(ref lines) = init_long_lines {
                                // if lines.len() > 0 {
                                //     last_timestamp = lines.get(lines.len()-1).unwrap().timestamp;
                                // }
                                
                                tokio::select! {
                                    _ = token.cancelled() => {
                                        return;
                                    }
                                    _ = client.send_to_client(
                                            ServerToClientEvent::LinesHistory(
                                                ChannelType::LinesHistory,
                                                lines.clone(),
                                                ticker.clone(),
                                                MarketType::Long
                                            )
                                        ) => {}
                                };
                            }

                            loop {
                                let last_timestamp = Utc::now();
                                let last_minute = last_timestamp.timestamp() - (last_timestamp.timestamp() % TimeFrame::One.to_secs_i64());

                                tokio::select! {
                                    _ = token.cancelled() => {
                                        break;
                                    }

                                    result = spread_rx.recv() => {
                                        match result {
                                            Ok((pair, _, spread)) => {
                                                if pair != exchange_pair.long_pair {
                                                    continue;
                                                }

                                                let now = Utc::now();
                                                let ts = now.timestamp();
                                                let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());
                                                
                                                // println!("{}; {}", start_minute, last_minute);

                                                if start_minute == last_minute {
                                                    client.send_to_client(
                                                        ServerToClientEvent::UpdateLine(
                                                            ChannelType::UpdateLine,
                                                            Line { 
                                                                timestamp: Utc.timestamp_opt(start_minute, 0).unwrap(), 
                                                                exchange_pair: pair, 
                                                                symbol: ticker.clone(), 
                                                                timeframe: TimeFrame::One, 
                                                                value: BigDecimal::from_str(&format!("{}", spread)).unwrap() 
                                                            },
                                                            MarketType::Long
                                                        )
                                                    ).await;
                                                } 
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                }
                            }
                        }
                    });
                }

                if !exchange_pair.short_pair.is_empty() {
                    let init_short_lines = get_spread_history(&pool, &ticker, &exchange_pair.short_pair).await;
                    tokio::spawn({
                        let ticker = ticker.clone();
                        let token = token.clone();
                        let mut client = client.clone();
                        let mut spread_rx = spread_tx.subscribe();

                        if let Ok(ref lines) = init_short_lines {
                            tokio::select! {
                                _ = token.cancelled() => {
                                    return;
                                }
                                _ = client.send_to_client(
                                        ServerToClientEvent::LinesHistory(
                                            ChannelType::LinesHistory,
                                            lines.clone(),
                                            ticker.clone(),
                                            MarketType::Short
                                        )
                                    ) => {}
                            };
                        }
                        
                        async move {
                            loop {
                                let last_timestamp = Utc::now();
                                let last_minute = last_timestamp.timestamp() - (last_timestamp.timestamp() % TimeFrame::One.to_secs_i64());

                                tokio::select! {
                                    _ = token.cancelled() => {
                                        break;
                                    }

                                    result = spread_rx.recv() => {
                                        match result {
                                            Ok((pair, _, spread)) => {
                                                if pair != exchange_pair.short_pair {
                                                    continue;
                                                }

                                                let now = Utc::now();
                                                let ts = now.timestamp();
                                                let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());
                                                
                                                // println!("{}; {}", start_minute, last_minute);

                                                if start_minute == last_minute {
                                                    client.send_to_client(
                                                        ServerToClientEvent::UpdateLine(
                                                            ChannelType::UpdateLine,
                                                            Line { 
                                                                timestamp: Utc.timestamp_opt(start_minute, 0).unwrap(), 
                                                                exchange_pair: pair, 
                                                                symbol: ticker.clone(), 
                                                                timeframe: TimeFrame::One, 
                                                                value: BigDecimal::from_str(&format!("{}", spread)).unwrap() 
                                                            },
                                                            MarketType::Short
                                                        )
                                                    ).await;
                                                } 
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                }
                            }
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
                                    MarketType::Long, 
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
                                    MarketType::Short, 
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