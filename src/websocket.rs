use std::{sync::Arc};

use futures_util::{StreamExt, SinkExt};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::exchanges;
use exchanges::orderbook::OrderbookLocal;

#[derive(Deserialize, Debug, Clone)]
pub struct WebsocketReceiverParams {
    pub exchange: String,
    pub symbol: String,
    pub order_type: String,
}

type UserMessageParams = Arc<RwLock<Option<WebsocketReceiverParams>>>;

pub async fn connect_async(book: Arc<RwLock<OrderbookLocal>>) {
    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await.unwrap();
    let params: UserMessageParams = Arc::new(RwLock::new(None));
    
    println!("🌐 [Arbitration-Websocket] is running",);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, book.clone(), params.clone()));
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream, 
    book: Arc<RwLock<OrderbookLocal>>,
    params: UserMessageParams
) {
    let ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream> = accept_async(stream).await.unwrap();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    println!("🟢 [Arbitration-Websocket] Client connected");

    let params = params.clone();
    tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            if let Ok(msg) = msg {
                if msg.is_text() {
                    if let Ok(new_params) = serde_json::from_str::<WebsocketReceiverParams>(&msg.to_text().unwrap()) {
                        println!("Получены новые параметры: {:?}", new_params);
                        *params.write().await = Some(new_params);
                    }
                }
            }
        }
    });

    loop {
        let book = book.read().await;
        let json = serde_json::to_string(&*book).unwrap();

        if ws_sender.send(Message::Text(json.into())).await.is_err() {
            println!("Client disconnected");
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}