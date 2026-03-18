use std::{collections::{HashMap, VecDeque}, sync::Arc, time::Duration};
use futures_util::{StreamExt, SinkExt};
use tokio::{net::TcpListener, sync::{mpsc, watch}};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::info;
use uuid::Uuid;

use crate::{models::{aggregator::{AggregatorPayload, ClientAggregatorUse}, line::{Line, MarketType}, websocket::{ChannelSubscription, ChannelType, ChartData, ChartEvent, ClientCmd, OrderBookData, Subscription}}, transport::client_aggregator::ClientAggregatorCmd};

const PING_DELAY: u64 = 20; // в секундах
const WEBSOCKET_NAME: &'static str = "ArbitrationWebsocket";

pub async fn connect_async(
    sender: mpsc::Sender<ClientAggregatorCmd>,
) {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await.unwrap();
    
    info!("{} -> is running", WEBSOCKET_NAME);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(
            stream, 
            sender.clone(),
        ));
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, 
    sender: mpsc::Sender<ClientAggregatorCmd>,
) {

    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let new_id = Uuid::new_v4();
    let (tx, mut rx) = watch::channel(Arc::new(AggregatorPayload::default()));
    let (lines_tx, mut lines_rx) = mpsc::channel::<(VecDeque<Line>, MarketType)>(1);
    let cancel_token = tokio_util::sync::CancellationToken::new();

    sender.send(
        ClientAggregatorCmd::Register { 
            client_id: new_id, 
            tx: tx.clone(),
            lines_tx
        }
    ).await.ok();

    tokio::spawn({
        let sender = sender.clone(); 
        let cancel_token = cancel_token.clone();

        let mut chart_data = HashMap::new();
        chart_data.insert(
            ChannelType::Chart, 
            ChartData::new()
        );

        let mut books = HashMap::new();
        books.insert(
            ChannelType::OrderBook,
            OrderBookData::new()
        );

        let mut interval = tokio::time::interval(Duration::from_millis(100));
        let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_DELAY));

        async move {
            loop {
                tokio::select! {
                    Some((lines, market_type)) = lines_rx.recv() => {
                        if let Some(chart) = chart_data.get_mut(&ChannelType::Chart) {
                            match market_type {
                                MarketType::Long => {
                                    let long_lines = lines
                                        .into_iter()
                                        .map(|line| {
                                            serde_json::json!({
                                                "time": line.timestamp,
                                                "value": line.value
                                            })
                                        })
                                        .collect();
                                    chart.long_lines = Some(long_lines);
                                },
                                MarketType::Short => {
                                    let short_lines = lines
                                        .into_iter()
                                        .map(|line| {
                                            serde_json::json!({
                                                "time": line.timestamp,
                                                "value": line.value
                                            })
                                        })
                                        .collect();
                                    chart.short_lines = Some(short_lines);
                                }
                            }
                        }
                    },
                    Ok(_) = rx.changed() => {
                        let payload = rx.borrow().clone();
                        
                        match payload.as_ref() {
                            AggregatorPayload::OrderBook { 
                                long_order_book,
                                short_order_book,
                                ticker,
                            } => {
                                if let Some(data) = books.get_mut(&ChannelType::OrderBook) {
                                    data.ticker = Some(ticker.clone());
                                    data.long = Some(long_order_book.clone());
                                    data.short = Some(short_order_book.clone());
                                }
                            },
                            AggregatorPayload::ChartEvent {
                                event,
                                ticker
                            } => {
                                if let Some(chart) = chart_data.get_mut(&ChannelType::Chart) {
                                    chart.ticker = Some(ticker.clone());
                                }

                                match event {
                                    ChartEvent::Volume24hr(
                                        long_vol, 
                                        short_vol
                                    ) => {
                                        if let Some(chart) = chart_data.get_mut(&ChannelType::Chart) {
                                            chart.volume24h = Some((*long_vol, *short_vol));
                                        }
                                    },
                                    ChartEvent::UpdateLine(
                                        long_spread, 
                                        short_spread,
                                        timestamp
                                    ) => {
                                        if let Some(chart) = chart_data.get_mut(&ChannelType::Chart) {
                                            chart.update_line = Some((
                                                *long_spread, 
                                                *short_spread,
                                                *timestamp
                                            ));
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        sender.send(ClientAggregatorCmd::Use(ClientAggregatorUse::UnRegister(new_id))).await.ok();
                        info!("{} -> отключился", new_id);
                        break;
                    },
                    _ = interval.tick() => {
                        for (key, data) in &books {
                            let long_book = data.long.clone();
                            let short_book = data.short.clone();

                            if let (Some(l), Some(s)) = (long_book, short_book) {
                                let msg = serde_json::json!({
                                    "channel": key,
                                    "result": {
                                        "order_book": {
                                            "long": l,
                                            "short": s
                                        }
                                    },
                                    "ticker": data.ticker
                                });

                                if ws_sender.send(Message::Text(msg.to_string())).await.is_err() {
                                    cancel_token.cancel();
                                }  
                            }
                        }
                        
                        for (key, data) in &chart_data {
                            let volume24h = data.volume24h;
                            let update_line = data.update_line;
                            let long_lines = &data.long_lines;
                            let short_lines = &data.short_lines;

                            let mut long_lines_vec = Vec::new();
                            let mut short_lines_vec = Vec::new();

                            if let (Some(long_lines), Some(short_lines)) = (long_lines, short_lines) {
                                long_lines_vec = long_lines.to_vec();
                                short_lines_vec = short_lines.to_vec();
                            }

                            let mut long_volume = 0.0;
                            let mut short_volume = 0.0;

                            if let Some((long_vol, short_vol)) = volume24h {
                                long_volume = long_vol;
                                short_volume = short_vol;
                            }

                            let mut updated_line = (0.0, 0.0, 0);
                            
                            if let Some((long_spread, short_spread, time)) = update_line {
                                updated_line = (long_spread, short_spread, time);
                            }

                            let msg = serde_json::json!({
                                "channel": key,
                                "result": {
                                    "lines": {
                                        "long": long_lines_vec,
                                        "short": short_lines_vec,
                                    },
                                    "events": {
                                        "volume24h": {
                                            "long_vol": long_volume,
                                            "short_vol": short_volume
                                        },
                                        "update_line": {
                                            "long": {
                                                "time": updated_line.2,
                                                "value": updated_line.0,
                                            },
                                            "short": {
                                                "time": updated_line.2,
                                                "value": updated_line.1
                                            },
                                        }
                                    }
                                },
                                "ticker": data.ticker
                            });

                            if ws_sender.send(Message::Text(msg.to_string())).await.is_err() {
                                cancel_token.cancel();
                            }
                        }
                    },
                    _ = ping_interval.tick() => {
                        if ws_sender.send(Message::Ping(vec![])).await.is_err() {
                            cancel_token.cancel();
                        }
                    }
                }
            }
        }
    });

    // Обрабатываем входящие сообщения от клиента
    while let Some(msg) = ws_receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(msg) => {
                    if let Ok(subscription) = serde_json::from_str::<Subscription>(&msg) {                        
                        let ticker = Arc::new(format!("{}usdt", subscription.ticker.to_lowercase()));

                        match subscription.action {
                            ClientCmd::Subscribe => {
                                match subscription.channel {
                                    ChannelType::OrderBook => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        sender.send(ClientAggregatorCmd::Use(
                                            ClientAggregatorUse::Subscribe(
                                                new_id, 
                                                ChannelSubscription::OrderBook { 
                                                    long_exchange, 
                                                    short_exchange, 
                                                    ticker: ticker
                                                }
                                            )
                                        )).await.ok();
                                    },
                                    ChannelType::Chart => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        sender.send(ClientAggregatorCmd::Use(
                                            ClientAggregatorUse::Subscribe(
                                                new_id, 
                                                ChannelSubscription::Chart { 
                                                    long_exchange, 
                                                    short_exchange, 
                                                    ticker: ticker
                                                }
                                            )
                                        )).await.ok();
                                    },
                                    _ => {}
                                }
                            }
                            ClientCmd::UnSubscribe => {
                                break;
                            }
                        }
                    }
                },
                Message::Pong(_) => {
                    info!("{} -> Ответил", new_id)
                },
                _ => {}
            }
        }
    }
}