use std::{sync::Arc, time::Duration};

use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{select, sync::{RwLock, broadcast, mpsc}};
use uuid::Uuid;

use crate::{exchanges::orderbook::{OrderType, SnapshotUi}, websocket::ConnectedClient};

mod exchanges;
mod exchange;
mod websocket;

#[tokio::main]
async fn main() {
    let (sender_exchange_names, receiver_exchange_names) = async_channel::unbounded::<ConnectedClient>();

    // Запускаем Websockets
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
