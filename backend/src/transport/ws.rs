use std::{collections::HashMap, time::Duration};
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;
use tokio::{net::TcpListener, time::interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::models::{candle::Candle, exchange::ExchangeType, orderbook::{OrderType, SnapshotUi}, websocket::{ChannelType, ClientCmd, Subscription}};

#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub uuid: Uuid,
    pub ticker: String,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub sender: async_channel::Sender<(ChannelType, OrderType, SnapshotUi, String)>,
    pub receiver: async_channel::Receiver<(ChannelType, OrderType, SnapshotUi, String)>,
    pub token: tokio_util::sync::CancellationToken,
}

impl ConnectedClient {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded::<(ChannelType, OrderType, SnapshotUi, String)>();
        
        Self { 
            uuid: Uuid::new_v4(),
            ticker: String::new(),
            long_exchange: ExchangeType::Unknown, 
            short_exchange: ExchangeType::Unknown,
            sender: sender,
            receiver: receiver,
            token: tokio_util::sync::CancellationToken::new()
        }
    }

    pub fn update(&mut self, ticker: &str, long_exchange: ExchangeType, short_exchange: ExchangeType) {
        self.ticker = ticker.to_string();
        self.long_exchange = long_exchange;
        self.short_exchange = short_exchange;
    }

    pub async fn send_snapshot(
        &mut self, 
        channel: ChannelType, 
        order_type: OrderType, 
        snapshot: SnapshotUi, 
        ticker: String
    ) {
        self.sender.send((channel, order_type.clone(), snapshot, ticker)).await.expect("[ConnectedClient] Failed to send snapshot")
    }

    pub async fn send_user_candles(&mut self, candles: Vec<Candle>) {
        // for candle in candles {
        println!("Sending candle: {:#?}", candles);
        // }
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
            let mut json: Value = serde_json::json!({
                "channel": "unknown",
                "result": {},
                "ticker": "unknown"
            });

            let mut books = HashMap::new();
            let mut interval = interval(Duration::from_millis(50));        

            loop {
                tokio::select! {
                    _ = task_token.cancelled() => {
                        println!("Client: `{}` disconnected", client.uuid);
                        break;
                    }

                    Ok((
                        channel,
                        order_type,
                        snapshot, 
                        ticker
                    )) = receiver.recv() => {
                        match channel {
                            ChannelType::OrderBook => {
                                books.insert(order_type, snapshot);
                                json = serde_json::json!({
                                    "channel": channel,
                                    "result": {
                                        "books": books
                                    },
                                    "ticker": ticker
                                });
                            },
                            ChannelType::CandlesHistory => {
                                json = serde_json::json!({
                                    "channel": channel,
                                    "result": {
                                        "candles": books
                                    },
                                    "ticker": ticker
                                });
                            }
                        }
                    }

                    _ = interval.tick() => {
                        if books.is_empty() {
                            continue;
                        }

                        if ws_sender.send(Message::Text(json.to_string().into())).await.is_err() {
                            task_token.cancel();
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
                                    ChannelType::CandlesHistory => {
                                        println!("candles_history is offend")
                                    }
                                    _ => {}
                                }
                            }
                            ClientCmd::UnSubscribe => {
                                println!("{:?}", subscription);
                                task_token.cancel();
                                break;
                            }
                            _ => {}
                        }

                        sender.send(client.clone()).await.expect("[Arbitration-Websocket] Failed to send exchange names");
                    }
                }
            }
        }
    });

}