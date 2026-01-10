use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use std::io::Read;

pub async fn connect(ticker: &str, channel_type: &str) {
    let url = Url::parse("wss://contract.mexc.com/edge").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Mexc] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [Mexc-Websocket] is running");

    // let orderbook = format!();
    
    write.send(Message::Text(
        serde_json::json!({
            "method": "sub.depth", 
            "param": { 
                "symbol": "BTC_USDT" 
            } 
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap();

        match data {
            Message::Text(text) => {
                println!("{text}")
            }
            _ => {}
        }
        
        // let json: OrderbookResponse = serde_json::from_str(&data_string).unwrap();

    });

    read_future.await;
}