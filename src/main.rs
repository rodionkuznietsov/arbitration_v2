use std::{time::Duration};
use crate::{websocket::ConnectedClient};

mod exchanges;
mod exchange;
mod websocket;

#[tokio::main]
async fn main() {
    let (sender_exchange_names, receiver_exchange_names) = async_channel::unbounded::<ConnectedClient>();

    tokio::spawn({
        async move {
            exchange::run_websockets(
                receiver_exchange_names,
            ).await;
        }
    });
    
    tokio::spawn({
        async move {
            websocket::connect_async(
                sender_exchange_names,
            ).await;
        }
    });

    loop {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
