use std::{collections::HashMap, time::Duration};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tokio::{net::TcpListener, time::interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::{exchange::ExchangeType, exchanges::orderbook::{OrderType, SnapshotUi}};

#[derive(Deserialize, Debug, Clone)]
pub struct WebsocketReceiverParams {
    pub exchanges: Exchanges,
    pub ticker: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Exchanges {
    #[serde(rename="longExchange")]
    pub long_exchange: String,
    #[serde(rename="shortExchange")]
    pub short_exchange: String
}

#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub uuid: Uuid,
    pub ticker: String,
    pub long_exchange: ExchangeType,
    pub short_exchange: ExchangeType,
    pub sender: async_channel::Sender<(OrderType, SnapshotUi)>,
    pub receiver: async_channel::Receiver<(OrderType, SnapshotUi)>,
    pub token: tokio_util::sync::CancellationToken,
}

impl ConnectedClient {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded::<(OrderType, SnapshotUi)>();
        
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

    pub async fn send_snapshot(&mut self, order_type: OrderType, snapshot: SnapshotUi) {
        self.sender.send((order_type.clone(), snapshot)).await.expect("[ConnectedClient] Failed to send snapshot")
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
            let mut ticker = interval(Duration::from_millis(50));        
            loop {
                tokio::select! {
                    _ = task_token.cancelled() => {
                        println!("Client: `{}` disconnected", client.uuid);
                        break;
                    }

                    Ok((order_type, snapshot)) = receiver.recv() => {
                        match order_type {
                            OrderType::Long => { books.insert("long", snapshot); },
                            OrderType::Short => { books.insert("short", snapshot);}
                        }
                    }

                    _ = ticker.tick() => {
                        if books.is_empty() {
                            continue;
                        }

                        let json = match serde_json::to_string(&books) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };

                        if ws_sender.send(Message::Text(json.into())).await.is_err() {
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
                    if let Ok(new_params) = serde_json::from_str::<WebsocketReceiverParams>(&msg.to_text().unwrap()) {
                        let long_exchange = new_params.exchanges.long_exchange.to_lowercase();
                        let short_exchange = new_params.exchanges.short_exchange.to_lowercase();
                        let ticker = new_params.ticker.to_lowercase();
                                                
                        let long_exchange= match long_exchange.as_str() {
                            "binance" => ExchangeType::Binance,
                            "bybit" => ExchangeType::Bybit,
                            "kucoin" => ExchangeType::KuCoin,
                            "binx" => ExchangeType::BinX,
                            "mexc" => ExchangeType::Mexc,
                            _ => ExchangeType::Unknown
                        };

                        let short_exchange= match short_exchange.as_str() {
                            "binance" => ExchangeType::Binance,
                            "bybit" => ExchangeType::Bybit,
                            "kucoin" => ExchangeType::KuCoin,
                            "binx" => ExchangeType::BinX,
                            "mexc" => ExchangeType::Mexc,
                            _ => ExchangeType::Unknown
                        };

                        client.update(&ticker, long_exchange, short_exchange);

                        sender.send(client.clone()).await.expect("[Arbitration-Websocket] Failed to send exchange names");
                    }
                }
            }
        }
    });

}