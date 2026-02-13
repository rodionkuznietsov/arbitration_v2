use std::{collections::HashMap, time::Duration};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use serde_json::Value;
use tokio::{net::TcpListener, time::interval};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use crate::{exchanges::orderbook::{OrderType, SnapshotUi}, models::candle::Candle, services::market_manager::ExchangeType};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="camelCase")]
pub struct Subscription {
    action: String,
    channel: String,
    long_exchange: Option<String>, 
    short_exchange: Option<String>,
    ticker: String
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
    pub sender: async_channel::Sender<(OrderType, SnapshotUi, String)>,
    pub receiver: async_channel::Receiver<(OrderType, SnapshotUi, String)>,
    pub token: tokio_util::sync::CancellationToken,
}

impl ConnectedClient {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded::<(OrderType, SnapshotUi, String)>();
        
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

    pub async fn send_snapshot(&mut self, order_type: OrderType, snapshot: SnapshotUi, ticker: String) {
        self.sender.send((order_type.clone(), snapshot, ticker)).await.expect("[ConnectedClient] Failed to send snapshot")
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
            let mut books = HashMap::new();
            let mut ticker = String::new();
            let mut interval = interval(Duration::from_millis(50));        
            loop {
                tokio::select! {
                    _ = task_token.cancelled() => {
                        println!("Client: `{}` disconnected", client.uuid);
                        break;
                    }

                    Ok((order_type, snapshot, symbol)) = receiver.recv() => {
                        ticker = symbol;
                        match order_type {
                            OrderType::Long => { books.insert("long", snapshot); },
                            OrderType::Short => { books.insert("short", snapshot);}
                        }
                    }

                    _ = interval.tick() => {
                        if books.is_empty() {
                            continue;
                        }

                        let json: Value = serde_json::json!({
                            "channel": "order_book",
                            "result": {
                                "books": books
                            },
                            "ticker": ticker
                        });

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
                        
                        match subscription.action.as_str() {
                            "subscribe" => {
                                match subscription.channel.as_str() {
                                    "order_book" => {
                                        let long_exchange = subscription.long_exchange.unwrap().to_lowercase();
                                        let short_exchange = subscription.short_exchange.unwrap().to_lowercase();

                                        let long_exchange= match long_exchange.as_str() {
                                            "binance" => ExchangeType::Binance,
                                            "bybit" => ExchangeType::Bybit,
                                            "kucoin" => ExchangeType::KuCoin,
                                            "binx" => ExchangeType::BinX,
                                            "mexc" => ExchangeType::Mexc,
                                            "gate" => ExchangeType::Gate,
                                            "lbank" => ExchangeType::LBank,
                                            _ => ExchangeType::Unknown
                                        };

                                        let short_exchange= match short_exchange.as_str() {
                                            "binance" => ExchangeType::Binance,
                                            "bybit" => ExchangeType::Bybit,
                                            "kucoin" => ExchangeType::KuCoin,
                                            "binx" => ExchangeType::BinX,
                                            "mexc" => ExchangeType::Mexc,
                                            "gate" => ExchangeType::Gate,
                                            "lbank" => ExchangeType::LBank,
                                            _ => ExchangeType::Unknown
                                        };

                                        client.update(&ticker, long_exchange, short_exchange);
                                    },
                                    "candles_history" => {
                                        println!("candles_history is offend")
                                    }
                                    _ => {}
                                }
                            }
                            "ubsubscribe" => {
                                println!("unsubscribe");
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