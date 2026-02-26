use std::{str::FromStr, sync::Arc};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::types::BigDecimal;
use tokio::sync::{broadcast, mpsc};

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{exchange::ExchangeType, line::{Line, TimeFrame}, orderbook::{MarketType, SnapshotUi}, websocket::{ChannelType, ChartEvent, ServerToClientEvent}}, services::{aggregator::Aggregator, volume24hr::ExchangeVolume}, storage::line_storage::get_spread_history, transport::ws::ConnectedClient};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)>;
    fn spawn_quote_updater(self: Arc<Self>);
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>);
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
    pool: sqlx::PgPool
) {
    let (aggregator_tx, aggregator_rx) = mpsc::channel(100);
    let (spread_tx, _) = broadcast::channel(1000);

    let aggregator = Aggregator::new(
        aggregator_rx, 
        spread_tx.clone(),
        pool.clone()
    );
    tokio::spawn(aggregator.run());

    let kucoin_websocket = KuCoinWebsocket::new(false);
    let bybit_websocket = BybitWebsocket::new(true, aggregator_tx.clone());
    let binx_websocket = BinXWebsocket::new(false);
    let mexc_websocket = MexcWebsocket::new(false);
    let binance_websocket = BinanceWebsocket::new(false);
    let gate_websocket = GateWebsocket::new(true, aggregator_tx.clone());
    let lbank_websocket = LBankWebsocket::new(false);

    let volume = ExchangeVolume::new(
        bybit_websocket.clone(),
        gate_websocket.clone(),
    );
    volume.spawn_volume_engine();
    
    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let exchange_pair = client.exchange_pair.clone();
        let ticker = client.ticker.clone();
        let client = client.clone();
        let volume_tx = volume.volume_tx.clone();

        // Volume24hr
        tokio::spawn({
            let mut volume_rx = volume_tx.subscribe();
            let mut client = client.clone();
            let token = token.clone();
            let chart_events = client.clone().chart_events;

            async move {
                if let Some(_) = chart_events.get(&ChartEvent::Volume24hr) {
                    loop {
                        match volume_rx.recv().await {
                            Ok((exchange_type, ticker, volume24hr)) => {
                                if long_exchange == exchange_type {
                                    if client.ticker != ticker {
                                        continue;
                                    }

                                    client.send_to_client(
                                        ServerToClientEvent::Volume24hr(
                                            ChartEvent::Volume24hr,
                                            ticker.clone(),
                                            volume24hr, 
                                            MarketType::Long
                                        )
                                    ).await
                                }

                                if short_exchange == exchange_type {
                                    if client.ticker != ticker {
                                        continue;
                                    }
                                    
                                    client.send_to_client(
                                        ServerToClientEvent::Volume24hr(
                                            ChartEvent::Volume24hr,
                                            ticker.clone(),
                                            volume24hr, 
                                            MarketType::Short
                                        )
                                    ).await
                                }
                            },
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_n)) => {
                                continue;
                            },
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                token.cancel();
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Инициализация последних 100 свечей
        tokio::spawn({
            let client = client.clone();
            let token = token.clone();
            let ticker = ticker.clone();
            let pool = pool.clone();
            let spread_tx = spread_tx.clone();

            async move {
                if let Some(_) = client.chart_events.get(&ChartEvent::UpdateLine) {
                    if !exchange_pair.long_pair.is_empty() {
                        let init_long_lines = get_spread_history(&pool, &ticker, &exchange_pair.long_pair).await;
                        tokio::spawn({
                            let ticker = ticker.clone();
                            let token = token.clone();
                            let mut client = client.clone();
                            let mut spread_rx = spread_tx.subscribe();

                            async move {
                                if let Ok(ref lines) = init_long_lines {
                                    tokio::select! {
                                        _ = token.cancelled() => {
                                            return;
                                        }
                                        _ = client.send_to_client(
                                                ServerToClientEvent::LinesHistory(
                                                    ChannelType::Chart,
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

                                    match spread_rx.recv().await {
                                        Ok((pair, ticker, spread)) => {
                                            if pair != exchange_pair.long_pair {
                                                continue;
                                            }

                                            if format!("{}usdt", client.ticker) != ticker {
                                                continue;
                                            }

                                            let now = Utc::now();
                                            let ts = now.timestamp();
                                            let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());
                                            
                                            if start_minute == last_minute {
                                                client.send_to_client(
                                                    ServerToClientEvent::UpdateLine(
                                                        ChartEvent::UpdateLine,
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
                                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                            continue;
                                        }
                                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                            token.cancel();
                                            break;
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
                                                ChannelType::Chart,
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

                                    match spread_rx.recv().await {
                                        Ok((pair, ticker, spread)) => {
                                            if pair != exchange_pair.short_pair {
                                                continue;
                                            }

                                            if format!("{}usdt", client.ticker) != ticker {
                                                continue;
                                            }

                                            let now = Utc::now();
                                            let ts = now.timestamp();
                                            let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());
                                            
                                            if start_minute == last_minute {
                                                client.send_to_client(
                                                    ServerToClientEvent::UpdateLine(
                                                        ChartEvent::UpdateLine,
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
                                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                            continue;
                                        }
                                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                            token.cancel();
                                            break;
                                        }
                                    }
                                }
                            }
                        });
                    }
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