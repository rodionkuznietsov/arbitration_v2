use std::{collections::{HashMap}, sync::Arc, time::Duration};
use futures_util::{StreamExt, SinkExt};
use tokio::{net::TcpListener, sync::{mpsc}};
use tokio_tungstenite::{accept_async, tungstenite::{Message, protocol::CloseFrame}};
use tracing::info;
use uuid::Uuid;

use crate::{models::{aggregator::{ClientAggregatorUse, KeyMarketType}, websocket::{ChannelSubscription, ChannelType, ClientCmd, ClientData, Subscription, WsClientMessage}}, transport::client_aggregator::ClientAggregatorCmd};

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
    let (orderbook_tx, mut orderbook_rx) = mpsc::channel::<Arc<WsClientMessage>>(100);
    let (lines_tx, mut lines_rx) = mpsc::channel::<Arc<WsClientMessage>>(100);
    let cancel_token = tokio_util::sync::CancellationToken::new();

    let (error_tx, mut error_rx) = mpsc::channel(100);

    sender.send(
        ClientAggregatorCmd::Register { 
            client_id: new_id, 
            tx: orderbook_tx.clone(),
            lines_tx
        }
    ).await.ok();

    tokio::spawn({
        let sender = sender.clone(); 
        let cancel_token = cancel_token.clone();

        let mut chart_data = HashMap::new();
        chart_data.insert(
            ChannelType::Chart, 
            ClientData::new()
        );

        let mut books = HashMap::new();
        books.insert(
            ChannelType::OrderBook,
            ClientData::new()
        );

        let mut interval = tokio::time::interval(Duration::from_millis(100));
        let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_DELAY));

        async move {
            loop {
                tokio::select! {
                    Some(error_msg) = error_rx.recv() => {
                        ws_sender.send(error_msg).await.ok();
                    }

                    Some(payload) = lines_rx.recv() => {
                        if let Some(data) = chart_data.get_mut(&ChannelType::Chart) {
                            let key = &payload.result.unique_id;
                            data.result.insert(key.clone(), payload);
                        }
                    },
                    Some(payload) = orderbook_rx.recv() => {
                        if let Some(data) = books.get_mut(&ChannelType::OrderBook) {
                            let key = &payload.result.unique_id;
                            data.result.insert(key.clone(), payload);
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        sender.send(ClientAggregatorCmd::Use(ClientAggregatorUse::UnRegister(new_id))).await.ok();
                        info!("{} -> отключился", new_id);
                        break;
                    },
                    _ = interval.tick() => {
                        for data in books.values() {
                            for result in data.result.values() {
                                let msg = serde_json::to_string(result).unwrap();

                                if ws_sender.send(Message::Text(msg.to_string())).await.is_err() {
                                    cancel_token.cancel();
                                }  
                            }
                        }
                        
                        for data in chart_data.values() {
                            for result in data.result.values() {
                                let msg = serde_json::to_string(result).unwrap();

                                if ws_sender.send(Message::Text(msg.to_string())).await.is_err() {
                                    cancel_token.cancel();
                                }  
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
                        
                        // Возращаем ошибку
                        if subscription.long_exchange == subscription.short_exchange {
                            error_tx.send(Message::Close(
                                Some(CloseFrame {
                                    code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Policy,
                                    reason: "invalid_subscirptions".into()
                                })
                            )).await.ok();
                            return ;
                        }

                                         
                        let ticker = Arc::new(format!("{}usdt", subscription.ticker.to_lowercase()));

                        match subscription.action {
                            ClientCmd::Subscribe => {
                                match subscription.channel {
                                    ChannelType::OrderBook => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        sender.send_timeout(ClientAggregatorCmd::Use(
                                            ClientAggregatorUse::Subscribe(
                                                new_id, 
                                                ChannelSubscription::OrderBook { 
                                                    long_market_type: KeyMarketType::new(
                                                        long_exchange, 
                                                        short_exchange, 
                                                        ticker.clone()
                                                    ), 
                                                    short_market_type: KeyMarketType::new(
                                                        short_exchange, 
                                                        long_exchange, 
                                                        ticker.clone()
                                                    ), 
                                                }
                                            )
                                        ), Duration::from_millis(10)).await.ok();
                                    },
                                    ChannelType::Chart => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        sender.send_timeout(
                                            ClientAggregatorCmd::Use(
                                                ClientAggregatorUse::Subscribe(
                                                    new_id, 
                                                    ChannelSubscription::Chart { 
                                                        long_market_type: KeyMarketType::new(
                                                            long_exchange, 
                                                            short_exchange, 
                                                            ticker.clone()
                                                        ), 
                                                        short_market_type: KeyMarketType::new(
                                                            short_exchange, 
                                                            long_exchange, 
                                                            ticker.clone()
                                                        ), 
                                                    }
                                                )
                                            ),
                                            Duration::from_millis(10)
                                        ).await.ok();
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