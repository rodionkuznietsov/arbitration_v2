use std::{collections::HashMap, num::NonZeroUsize, time::Duration};
use futures_util::{StreamExt, SinkExt};
use lru::LruCache;
use serde_json::Value;
use tokio::{net::TcpListener, sync::mpsc, time::interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::models::{line::Line, exchange::ExchangeType, websocket::{ChannelType, ClientCmd, ServerToClientEvent, Subscription}};

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
    pub exchange_pair: String
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

            candles, exchange_pair: String::new()
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
                                candles,
                                ticker
                            ) => {

                                let candles_json: Vec<Value> = candles.iter().map(|c| {
                                    serde_json::json!({
                                        "timestamp": c.timestamp.to_rfc3339(),
                                        "exchange_pair": c.exchange_pair,
                                        "symbol": c.symbol,
                                        "timeframe": c.timeframe,
                                        "value": c.value.to_string()
                                    })
                                }).collect();

                                let json = serde_json::json!({
                                    "channel": channel,
                                    "result": {
                                        "lines": candles_json
                                    },
                                    "ticker": ticker
                                });

                                lines_history.insert(channel, json);
                            },
                            ServerToClientEvent::UpdateHistory(channel, line) => {
                                println!("{:?}, {:?}", channel, line);

                                let line_json: Value = serde_json::json!({
                                    "timestamp": line.timestamp.to_rfc3339(),
                                    "exchange_pair": line.exchange_pair,
                                    "symbol": line.symbol,
                                    "timeframe": line.timeframe,
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

                                print!("{:?}", lines_history);
                            }
                            ServerToClientEvent::UpdateLine(channel, line, ticker) => {
                                let line_json = serde_json::json!({
                                    "timestamp": line.timestamp.to_rfc3339(),
                                    "exchange_pair": line.exchange_pair,
                                    "symbol": line.symbol,
                                    "timeframe": line.timeframe,
                                    "value": line.value.to_string()
                                });

                                let event = serde_json::json!({
                                    "events": {
                                        "event": channel,
                                        "line": line_json
                                    },
                                });

                                lines_history.entry(ChannelType::LinesHistory)
                                    .and_modify(|json| {
                                        if let Some(result) = json.get_mut("result").and_then(|v| v.as_object_mut()) {
                                            result.insert("events".to_string(), event["events"].clone());
                                        }
                                    })
                                    .or_insert_with(|| {
                                        serde_json::json!({ "result": event  })
                                    });
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

                                        let pair = format!("{}/{}", long_exchange, short_exchange);
                                        client.exchange_pair = pair;
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