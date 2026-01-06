use std::sync::{Arc};

use futures_util::{StreamExt, SinkExt};
use tokio::{net::TcpListener, sync::RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::exchanges::bybit::OrderbookLocal;

pub async fn connect_async(book: Arc<RwLock<OrderbookLocal>>) {
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    println!("🌐 [Arbitration-Websocket] is running",);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, book.clone()));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, book: Arc<RwLock<OrderbookLocal>>) {
    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_stream) = ws_stream.split();
    println!("🟢 [Arbitration-Websocket] Client connected");

    loop {
        let book = book.read().await;
        let json = serde_json::to_string(&*book).unwrap();

        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            println!("Client disconnected");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}