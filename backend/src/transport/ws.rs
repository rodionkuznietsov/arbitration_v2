use std::{collections::HashMap, num::NonZeroUsize, time::Duration};
use dashmap::DashMap;
use futures_util::{StreamExt, SinkExt};
use lru::LruCache;
use serde_json::Value;
use tokio::{net::TcpListener, time::interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::models::{exchange::{ExchangePairs, ExchangeType}, line::Line, orderbook::MarketType, websocket::{ChannelType, ClientCmd, ServerToClientEvent, Subscription}};

#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub uuid: Uuid,
    pub ticker: String,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub sender: async_channel::Sender<ServerToClientEvent>,
    pub receiver: async_channel::Receiver<ServerToClientEvent>,
    pub token: tokio_util::sync::CancellationToken,
    
    pub candles: LruCache<String, Line>,
    pub exchange_pair: ExchangePairs,
}

impl ConnectedClient {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::bounded::<ServerToClientEvent>(5);
        let candles = LruCache::new(NonZeroUsize::new(100).unwrap());

        Self { 
            uuid: Uuid::new_v4(),
            ticker: String::new(),
            long_exchange: ExchangeType::Unknown, 
            short_exchange: ExchangeType::Unknown,
            sender: sender,
            receiver: receiver,
            token: tokio_util::sync::CancellationToken::new(),

            candles, exchange_pair: ExchangePairs::new(),
        }
    }

    pub fn update(&mut self, ticker: &str, long_exchange: ExchangeType, short_exchange: ExchangeType) {
        self.ticker = ticker.to_string();
        self.long_exchange = long_exchange;
        self.short_exchange = short_exchange;
    }

    pub async fn send_to_client(
        &mut self, 
        event: ServerToClientEvent
    ) {
        self.sender.send(event).await.expect("[ConnectedClient] Failed to send snapshot")
    }
}

pub async fn connect_async(
    sender: async_channel::Sender<ConnectedClient>,
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
    sender: async_channel::Sender<ConnectedClient>,
) {

    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut client = ConnectedClient::new();
    let receiver = client.receiver.clone(); 
    let task_token = client.token.clone();

    println!("Client: `{}` is successfully connected", client.uuid);

    tokio::spawn({
        async move {
            let mut books = HashMap::new();
            let mut orderbooks = HashMap::new();
            let mut lines_history = HashMap::new();
            let mut interval = interval(Duration::from_millis(50));   

            loop {
                tokio::select! {
                    _ = task_token.cancelled() => {
                        println!("Client: `{}` disconnected", client.uuid);
                        break;
                    }

                    Ok(event) = receiver.recv() => {
                        match event {
                            ServerToClientEvent::OrderBook(
                                channel, 
                                order_type, 
                                snapshot, 
                                ticker
                            ) => {
                                books.insert(order_type, snapshot);
                                let json = serde_json::json!({
                                    "channel": channel,
                                    "result": {
                                        "books": books
                                    },
                                    "ticker": ticker
                                });
                                orderbooks.insert(channel, json);
                            }
                            ServerToClientEvent::LinesHistory(
                                channel, 
                                lines,
                                ticker,
                                market_type
                            ) => {

                                let entry = lines_history
                                    .entry(channel.clone())
                                    .or_insert_with(|| {
                                            serde_json::json!({
                                                "channel": channel,
                                                "result": {
                                                    "lines": {}
                                                },
                                                "ticker": ticker,
                                            })
                                        }
                                    );

                                if let Some(result) = entry.get_mut("result") {
                                    let lines_obj = result
                                        .as_object_mut()
                                        .unwrap()
                                        .entry("lines")
                                        .or_insert_with(|| serde_json::json!({}));

                                    lines_obj[&market_type.to_string()] = serde_json::json!(
                                        lines.iter().map(|line| {
                                            serde_json::json!({
                                                "time": line.timestamp.to_rfc3339(),
                                                "value": line.value.to_string()                                        
                                            })
                                        }).collect::<Vec<_>>()
                                    );
                                }
                            },
                            ServerToClientEvent::UpdateHistory(_channel, line, market_type) => {
                                let line_json: Value = serde_json::json!({
                                    "time": line.timestamp.to_rfc3339(),
                                    "value": line.value.to_string()
                                });

                                lines_history.entry(ChannelType::LinesHistory)
                                    .and_modify(|json| {
                                        if let Some(result) = json.get_mut("result").and_then(|v| v.as_object_mut()) {
                                            if let Some(lines) = result.get_mut("lines").and_then(|v| v.as_array_mut()) {
                                                lines.push(line_json);
                                            }
                                        } 
                                    });
                            }
                            ServerToClientEvent::UpdateLine(channel, line, market_type) => {
                                let entry = lines_history
                                    .entry(ChannelType::LinesHistory)
                                    .or_insert_with(|| {
                                            serde_json::json!({
                                                "channel": channel,
                                                "result": {
                                                    "events": {
                                                        "update_line": {}
                                                    }
                                                },
                                            })
                                        }
                                    );

                                if let Some(result) = entry.get_mut("result") {
                                    let events = result
                                        .as_object_mut()
                                        .unwrap()
                                        .entry("events")
                                        .or_insert_with(|| serde_json::json!({}));
                                        
                                    let update_line = events
                                        .as_object_mut()
                                        .unwrap()
                                        .entry("update_line")
                                        .or_insert_with(|| serde_json::json!({}));

                                    if let Some(update_line_obj) = update_line.as_object_mut() {
                                        update_line_obj.insert(
                                            market_type.to_string(),
                                            serde_json::json!({
                                                "time": line.timestamp.to_rfc3339(),
                                                "value": line.value.to_string()
                                            })
                                        );
                                    }
                                }
                            }
                        }
                    }

                    _ = interval.tick() => {
                        for json in orderbooks.values() {
                            if ws_sender.send(Message::Text(json.to_string().into())).await.is_err() {
                                task_token.cancel();
                            }
                        }

                        for json in lines_history.values() {
                            // println!("{:#?}", json);
                            if ws_sender.send(Message::Text(json.to_string().into())).await.is_err() {
                                task_token.cancel();
                            }
                        }
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    println!("{}", msg);
                    if let Ok(subscription) = serde_json::from_str::<Subscription>(&msg.to_text().unwrap()) {                        
                        
                        let ticker = subscription.ticker.to_lowercase();
                        let task_token = client.token.clone();

                        match subscription.action {
                            ClientCmd::Subscribe => {
                                match subscription.channel {
                                    ChannelType::OrderBook => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        client.update(&ticker, long_exchange, short_exchange);
                                    },
                                    ChannelType::LinesHistory => {
                                        let long_exchange = subscription.long_exchange.unwrap();
                                        let short_exchange = subscription.short_exchange.unwrap();

                                        let long_pair = format!("{}/{}", long_exchange, short_exchange);
                                        let short_pair = format!("{}/{}", short_exchange, long_exchange);
                                        client.exchange_pair = ExchangePairs {
                                            long_pair: long_pair,
                                            short_pair: short_pair
                                        };
                                    },
                                    _ => {}
                                }
                            }
                            ClientCmd::UnSubscribe => {
                                println!("{:?}", subscription);
                                task_token.cancel();
                                break;
                            }
                        }

                        sender.send(client.clone()).await.expect("[Arbitration-Websocket] Failed to send exchange names");
                    }
                }
            }
        }
    });

}