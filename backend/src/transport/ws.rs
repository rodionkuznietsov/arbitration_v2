use std::{collections::{HashMap, HashSet}, sync::Arc, time::Duration};
use futures_util::{StreamExt, SinkExt};
use tokio::{net::TcpListener, sync::{mpsc}};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::models::{aggregator::{AggregatorPayload, ClientAggregatorCmd}, websocket::{ChannelSubscription, ChannelType, ChartEvent, ClientCmd, Subscription, WsMessage, WebsocketResult}};

pub async fn connect_async(
    sender: mpsc::Sender<ClientAggregatorCmd>,
) {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await.unwrap();
    
    println!("🌐 [Arbitration-Websocket] is running",);

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
    let (tx, mut rx) = mpsc::channel::<Arc<AggregatorPayload>>(1024);
    let cancel_token = tokio_util::sync::CancellationToken::new();

    sender.send(ClientAggregatorCmd::Register { 
        id: new_id, 
        tx: tx
    }).await.ok();

    tokio::spawn({
        let sender = sender.clone(); 
        let mut books = HashMap::new();

        let mut interval = tokio::time::interval(Duration::from_millis(20));

        async move {
            loop {
                tokio::select! {
                    Some(payload) = rx.recv() => {
                        let payload = payload.as_ref();
                        
                        match payload {
                            AggregatorPayload::OrderBook { 
                                long_order_book,
                                short_order_book,
                                ticker
                            } => {
                                let msg = WsMessage {
                                    channel: ChannelType::OrderBook,
                                    result: WebsocketResult::OrderBook { 
                                        long: long_order_book.clone(), 
                                        short: short_order_book.clone() 
                                    },
                                    ticker: ticker.clone()
                                };

                                let string_msg = serde_json::to_string(&msg).unwrap();

                                books.insert(ChannelType::OrderBook, string_msg);
                            },
                            _ => {}
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        sender.send(ClientAggregatorCmd::UnRegister(new_id)).await.ok();
                        println!("Client {} disconnected", new_id);
                        break;
                    },
                    _ = interval.tick() => {
                        for (_, str_msg) in &books {
                            if ws_sender.send(Message::Text(str_msg.to_string())).await.is_err() {
                                cancel_token.cancel();
                            }
                        }
                    }
                }
            }
        }
    });

    // Обрабатываем входящие сообщения от клиента
    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    if let Ok(subscription) = serde_json::from_str::<Subscription>(&msg.to_text().unwrap()) {                        
                        let ticker = Arc::new(format!("{}usdt", subscription.ticker.to_lowercase()));

                        match subscription.action {
                            ClientCmd::Subscribe => {
                                match subscription.channel {
                                    ChannelType::OrderBook => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        sender.send(
                                            ClientAggregatorCmd::Subscribe(
                                                new_id, 
                                                ChannelSubscription::OrderBook { 
                                                    long_exchange, 
                                                    short_exchange, 
                                                    ticker: ticker
                                                }
                                            )
                                        ).await.ok();
                                    },
                                    ChannelType::Chart => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        let mut events = HashSet::new();
                                        events.insert(ChartEvent::LinesHistory);
                                        events.insert(ChartEvent::UpdateHistory);
                                        events.insert(ChartEvent::UpdateLine);
                                        events.insert(ChartEvent::Volume24hr);

                                        sender.send(ClientAggregatorCmd::Subscribe(
                                            new_id, 
                                            ChannelSubscription::Chart { 
                                                long_exchange, 
                                                short_exchange, 
                                                ticker: ticker
                                            }
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
                }
            }
        }
    });
}