use std::{collections::HashMap, str::FromStr, sync::Arc};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures_util::future::join_all;
use sqlx::types::BigDecimal;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{exchange::{ExchangePairs, ExchangeType}, line::{Line, TimeFrame}, orderbook::{MarketType, SnapshotUi}, websocket::{ChannelSubscription, ChannelType, ChartEvent, ServerToClientEvent}}, services::{aggregator::{Aggregator, AggregatorCommand}}, transport::ws::ConnectedClient};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn spawn_quote_updater(
        self: Arc<Self>,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    );

    fn spawn_volume_updater(
        self: Arc<Self>,
        aggregator_tx: mpsc::Sender<AggregatorCommand>
    );
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
    pool: sqlx::PgPool
) {
    let (aggregator_tx, aggregator_rx) = mpsc::channel(100);
    let (spread_tx, _) = broadcast::channel(1000);
    let (lines_cache_tx, _) = broadcast::channel(1);
    let (books_tx, _) = broadcast::channel::<HashMap<(ExchangeType, String), SnapshotUi>>(1);
    let (volume_tx, _) = broadcast::channel::<(String, String, f64)>(1000);

    let aggregator = Aggregator::new(
        aggregator_rx, 
        spread_tx.clone(),
        pool.clone(),
        lines_cache_tx.clone(),
        books_tx.clone(),
        volume_tx.clone()
    );
    tokio::spawn(aggregator.run());

    BybitWebsocket::new(true, aggregator_tx.clone());
    GateWebsocket::new(true, aggregator_tx.clone());
    KuCoinWebsocket::new(false);
    BinXWebsocket::new(false);
    MexcWebsocket::new(false);
    BinanceWebsocket::new(false);
    LBankWebsocket::new(false);

    loop {
        if let Ok(client) = receiver.recv().await {
            let client = client.clone();
            let subscriptions = client.subscriptions.clone();
            let ticker = client.ticker.clone();
            let aggregator_tx = aggregator_tx.clone();
            let spread_tx = spread_tx.clone();

            for channel_subscriptions in subscriptions.values() {
                match channel_subscriptions {
                    ChannelSubscription::Chart { 
                        events,
                        exchange_pairs
                    } => {
                        let client = client.clone();

                        if events.contains(&ChartEvent::Volume24hr) {
                            let client = client.clone();
                            let token = client.token.clone();
                            let volume_tx = volume_tx.clone();
                            let ticker = ticker.clone();
                            let exchange_pairs = exchange_pairs.clone();

                            tokio::spawn(async move {
                                loop {
                                    tokio::select! {
                                        _ = token.cancelled() => {
                                                break;
                                            }
                                        
                                        _ = handle_volume(
                                            volume_tx.clone(),
                                            ticker.clone(),
                                            exchange_pairs.clone(),
                                            client.clone()
                                        ) => {}
                                    }
                                }
                            });
                        }

                        if events.contains(&ChartEvent::LinesHistory) {
                            let mut client = client.clone();
                            let futures = exchange_pairs.iter().map(|(market_type, pair)| {
                                let aggregator_tx = aggregator_tx.clone();
                                let ticker = ticker.clone();
                                
                                async move {
                                    let lines = init_lines(
                                        aggregator_tx, 
                                        pair, 
                                        ticker, 
                                    ).await;
                                    (market_type, lines)
                                }
                            }); 
                            let results = join_all(futures).await;

                            client.send_to_client(
                                ServerToClientEvent::LinesHistory(
                                   ChannelType::Chart,
                                   results,
                                   ticker.clone()
                                )
                            ).await;
                        } 
                        
                        if events.contains(&ChartEvent::UpdateHistory) {
                            tokio::spawn({
                                let exchange_pairs = exchange_pairs.clone();
                                let client = client.clone();
                                let ticker = client.ticker.clone();
                                let token = client.token.clone();
                                let lines_tx = lines_cache_tx.clone();
                                let client = client.clone();

                                async move {
                                    loop {
                                        tokio::select! {
                                            _ = token.cancelled() => {
                                                break;
                                            }

                                            _ = handle_lines_cache(
                                                lines_tx.clone(),
                                                exchange_pairs.clone(),
                                                ticker.clone(),
                                                client.clone()
                                            ) => {}
                                        }
                                    }
                                }
                            });
                        }

                        if events.contains(&ChartEvent::UpdateLine) {
                            tokio::spawn({
                                let client = client.clone();
                                let token = client.token.clone();
                                let spread_tx = spread_tx.clone();
                                let aggregator_tx = aggregator_tx.clone();
                                let ticker = ticker.clone();
                                let exchange_pairs = exchange_pairs.clone();

                                async move {
                                    let (tx, rx) = oneshot::channel();
                                    if aggregator_tx.send(
                                        AggregatorCommand::GetLastTimestamp { 
                                            exchange_pair: exchange_pairs.long_pair.clone(), 
                                            ticker: ticker.clone(), 
                                            reply: tx
                                        }
                                    ).await.is_err() {}

                                    let mut last_timestamp = 0;

                                    if let Ok(timestamp) = rx.await {
                                        last_timestamp = timestamp
                                    }

                                    loop {
                                        tokio::select! {
                                            _ = token.cancelled() => {
                                                break;
                                            }

                                            _ = update_line(
                                                spread_tx.clone(),
                                                client.clone(),
                                                exchange_pairs.clone(),
                                                ticker.clone(),
                                                last_timestamp
                                            ) => {
                                                
                                            } 
                                        }
                                    }
                                }
                            });
                        }
                    },
                    ChannelSubscription::OrderBook { 
                        long_exchange, 
                        short_exchange 
                    } => {
                        let tx = books_tx.clone();
                        let ticker = ticker.clone();
                        let long_exchange = long_exchange.clone();
                        let short_exchange = short_exchange.clone();
                        let client = client.clone();
                        let token = client.token.clone();

                        tokio::spawn(async move {
                            loop {
                                tokio::select! {
                                    _ = token.cancelled() => {
                                        break;
                                    }

                                    _ = handle_orderbooks(
                                        long_exchange,
                                        short_exchange,
                                        ticker.clone(),
                                        client.clone(),
                                        tx.clone()
                                    ) => {

                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}

async fn handle_volume(
    volume_tx: broadcast::Sender<(String, String, f64)>,
    ticker: String,
    exchange_pairs: ExchangePairs,
    mut client: ConnectedClient
) {
    let mut rx = volume_tx.subscribe();
    loop {    
        match rx.recv().await {
            Ok((_pair, t, volume)) => {
                if ticker != t {
                    continue;
                }
                
                for (market_type, _exchange_pair) in exchange_pairs.iter() {
                    if _pair == _exchange_pair {
                        client.send_to_client(ServerToClientEvent::Volume24hr(
                            ChartEvent::Volume24hr,
                            ticker.clone(),
                            volume,
                            market_type
                        )).await;
                    }
                }
            },
            Err(broadcast::error::RecvError::Closed) => {
                break;
            },
            _ => continue
        }
    }
}

async fn handle_orderbooks(
    long_exchange: ExchangeType,
    short_exchange: ExchangeType,
    ticker: String,
    mut client: ConnectedClient,
    books_tx: broadcast::Sender<HashMap<(ExchangeType, String), SnapshotUi>>
) {
    let mut rx = books_tx.subscribe();
    match rx.recv().await {
        Ok(books) => {
            if let Some(snapshot) = books.get(&(long_exchange, ticker.clone())) {
                client.send_to_client(
                    ServerToClientEvent::OrderBook(
                        ChannelType::OrderBook,
                        MarketType::Long,
                        snapshot.clone(),
                        ticker.clone()
                    )
                ).await;
            }

            if let Some(snapshot) = books.get(&(short_exchange, ticker.clone())) {
                client.send_to_client(
                    ServerToClientEvent::OrderBook(
                        ChannelType::OrderBook,
                        MarketType::Short,
                        snapshot.clone(),
                        ticker.clone()
                    )
                ).await;
            }
        },
        Err(broadcast::error::RecvError::Closed) => {
            return;
        },
        _ => {}
    }
}

async fn update_line(
    spread_tx: broadcast::Sender<(String, String, f64)>,
    mut client: ConnectedClient,
    exchange_pairs: ExchangePairs,
    ticker: String,
    mut last_timestamp: i64
) {
    let mut rx = spread_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok((pair, t_, spread)) => {      
                
                if ticker != t_ {
                    continue;
                }
                
                let now = Utc::now();
                let ts = now.timestamp();
                let start_minute = ts - (ts % TimeFrame::One.to_secs_i64());

                if start_minute > last_timestamp {
                    last_timestamp = start_minute;
                }

                if start_minute == last_timestamp {
                    for (market_type, p) in exchange_pairs.iter() {
                        if p == pair {                            
                            client.send_to_client(
                            ServerToClientEvent::UpdateLine(
                                    ChartEvent::UpdateLine, 
                                    Line { 
                                        timestamp: Utc.timestamp_opt(start_minute, 0).unwrap(), 
                                        exchange_pair: pair.clone(), 
                                        symbol: ticker.clone(), 
                                        timeframe: TimeFrame::One, 
                                        value: BigDecimal::from_str(&format!("{}", spread)).unwrap()
                                    }, 
                                    market_type
                                )
                            ).await;
                        }
                    }
                } 
            },
            Err(broadcast::error::RecvError::Closed) => {
                break;
            },
            Err(broadcast::error::RecvError::Lagged(_)) => {}
        }
    }
}

async fn init_lines(
    aggregator_tx: mpsc::Sender<AggregatorCommand>,
    exchange_pair: &str,
    ticker: String,
) -> Vec<Line> {
    let (tx, reply) = oneshot::channel();
    
    if aggregator_tx.send(
        AggregatorCommand::GetLinesHistory { 
            exchange_pair: exchange_pair.to_string(), 
            ticker: ticker.clone(), 
            reply: tx
        }
    ).await.is_err() {};

    if let Ok(lines) = reply.await {
        return lines;
    } else {
        vec![]
    }
}

async fn handle_lines_cache(
    lines_tx: broadcast::Sender<HashMap<(String, String), Vec<Line>>>,
    exchange_pairs: ExchangePairs,
    ticker: String,
    mut client: ConnectedClient
) {
    let mut lines_rx = lines_tx.subscribe();
   
    match lines_rx.recv().await {
        Ok(latest_lines) => {
            if let Some(line) = latest_lines.get(&(exchange_pairs.long_pair.clone(), format!("{}usdt", ticker.clone()))) {                    
                client.send_to_client(ServerToClientEvent::AddLineToHistory(
                    ChannelType::Chart,
                    line.clone(),
                    ticker.clone(),
                    MarketType::Long
                )).await;
            }

            if let Some(line) = latest_lines.get(&(exchange_pairs.short_pair.clone(), format!("{}usdt", ticker.clone()))) {
                client.send_to_client(ServerToClientEvent::AddLineToHistory(
                    ChannelType::Chart,
                    line.clone(),
                    ticker.clone(),
                    MarketType::Short
                )).await;
            }
        },
        Err(broadcast::error::RecvError::Closed) => {
            return;
        },
        Err(broadcast::error::RecvError::Lagged(_)) => {}
    }
}