use std::{time::Duration};

mod exchanges;
mod exchange;
mod websocket;

#[tokio::main]
async fn main() {

    tokio::spawn({
        async move {
            websocket::connect_async().await;
        } 
    });

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
