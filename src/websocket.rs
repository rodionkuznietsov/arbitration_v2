use std::{collections::{HashMap}, sync::Arc};
use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::{RwLock, broadcast, mpsc::{self, Receiver}}};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::exchanges::orderbook::{LocalOrderBook, OrderType, Snapshot, SnapshotUi};

#[derive(Deserialize, Debug, Clone)]
pub struct WebsocketReceiverParams {
    pub exchanges: Exchanges,
    pub types: OrderTypes,
    pub ticker: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Exchanges {
    #[serde(rename="longExchange")]
    pub long_exchange: String,
    #[serde(rename="shortExchange")]
    pub short_exchange: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct OrderTypes {
    #[serde(rename="longType")]
    pub long_type: String,
    #[serde(rename="shortType")]
    pub short_type: String
}

pub async fn connect_async(
    long_book: Arc<RwLock<LocalOrderBook>>, 
    short_book: Arc<RwLock<LocalOrderBook>>, 
    sender: mpsc::Sender<(String, String, String)>,
    snapshot_sender: broadcast::Sender<(OrderType, SnapshotUi)>
) {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await.unwrap();
    
    println!("🌐 [Arbitration-Websocket] is running",);

    while let Ok((stream, _)) = listener.accept().await {
        let snapshot_receiver = snapshot_sender.subscribe();
        tokio::spawn(handle_connection(
            stream, 
            long_book.clone(),
            short_book.clone(),
            sender.clone(),
            snapshot_receiver
        ));
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, 
    long_book: Arc<RwLock<LocalOrderBook>>,
    short_book: Arc<RwLock<LocalOrderBook>>,
    sender: mpsc::Sender<(String, String, String)>,
    mut snapshot_receiver: broadcast::Receiver<(OrderType, SnapshotUi)>
) {

    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    println!("🟢 [Arbitration-Websocket] Client connected");

    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    if let Ok(new_params) = serde_json::from_str::<WebsocketReceiverParams>(&msg.to_text().unwrap()) {
                        let long_exchange = new_params.exchanges.long_exchange.to_lowercase();
                        let short_exchange = new_params.exchanges.short_exchange.to_lowercase();
                        let ticker = new_params.ticker.to_lowercase();
                        
                        sender.send((ticker.to_lowercase(), long_exchange, short_exchange)).await.expect("[Arbitration-Websocket] Failed to send exchange names");
                    }
                }
            }
        }
    });

    loop {

        let mut long_snapshot = SnapshotUi {
            a: vec![],
            b: vec![],
            last_price: 0.0,
        };

        match snapshot_receiver.recv().await {
            Ok((order_type, snapshot_ui)) => {
                match order_type {
                    OrderType::Long => {
                        long_snapshot = snapshot_ui
                    },
                    OrderType::Short => {
                        println!("Short: {:?}", snapshot_ui)
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(_)) => {}
        }

        if long_snapshot.last_price != 0.0 {
            let mut books = HashMap::new();
            books.insert("long", long_snapshot.clone());
            books.insert("book2", long_snapshot.clone());

            let json = serde_json::to_string(&books).unwrap();

            if ws_sender.send(Message::Text(json.into())).await.is_err() {
                println!("Client disconnected");            
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
}