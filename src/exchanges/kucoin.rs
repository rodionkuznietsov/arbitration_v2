use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use anyhow::Result;

use crate::exchanges::orderbook::LocalOrderBook;

#[derive(Debug, Deserialize, Serialize)]
struct ApiKeyResponse {
    #[serde(rename="data")]
    data: Data
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    #[serde(rename="token")]
    token: String
}

#[derive(Debug, Deserialize, Serialize)]
struct Snapshot {
    #[serde(rename="data")]
    data: Option<SnapshotData>
}

#[derive(Debug, Deserialize, Serialize)]
struct SnapshotData {
    #[serde(rename="asks")]
    asks: Option<Vec<Vec<String>>>,
    #[serde(rename="bids")]
    bids: Option<Vec<Vec<String>>>,
    #[serde(rename="price")]
    price: Option<String>
}

pub async fn connect(ticker: &str, channel_type: &str, local_book: Arc<RwLock<LocalOrderBook>>) {
    let api_token = get_api_key().await.unwrap();
    
    let url = url::Url::parse(&format!("wss://ws-api-spot.kucoin.com?token={}", api_token)).unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("[KuCoin] Failed to connect");
    let (mut write, read) = ws_stream.split();

    println!("🌐 [KuCoin-Websocket] is running");

    let channel_type = {
        if channel_type == "фьючерс" {
            "futures";
        }
        "spot"
    };

    let orderbook = format!("/{}Market/level2Depth50:{}-USDT", channel_type.to_lowercase(), ticker.to_uppercase());
    // let price = format!("/market/ticker:{}-USDT", ticker.to_uppercase());

    write.send(Message::Text(
        serde_json::json!({
            "type": "subscribe",
            "topic": orderbook,
            "response": true
        }).to_string().into()
    )).await.unwrap();

    write.send(Message::Text(
        serde_json::json!({
            "type": "subscribe",
            "topic": "/market/ticker:BTC-USDT",
            "response": true
        }).to_string()
    )).await.unwrap();

    let read_future = read.for_each(|msg| {
        let value = local_book.clone();
        async move {
            let data = msg.unwrap();

            match data {
                Message::Text(txt) => {
                    let json = serde_json::from_str::<serde_json::Value>(&txt);
                    match json {
                        Ok(v) => {
                            let msg_type = v.get("type").and_then(|t| t.as_str());

                            if msg_type == Some("error") {
                                println!("{:?}", msg_type);
                                return;
                            }
                        }
                        Err(e) => {
                            println!("Error: {e}")
                        }
                    }
                }
                Message::Close(_) => {
                    return;
                }
                _ => {

                }
            }
            
            // let data_string = String::from_utf8(data.to_vec()).unwrap();
            // let data = serde_json::from_str::<Snapshot>(&data_string).unwrap();
            // let book = value.clone();

            // fetch_data(data, book.clone()).await;
        }
    });

    read_future.await;

    println!("Is returned");
}

async fn fetch_data(data: Snapshot, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        let current = local_book.read().await;
        current.clone()
    };

    if let Some(d) = data.data {
        if let Some(asks) = d.asks {
            for ask in asks {
                let price = ask[0].parse::<f64>().unwrap();
                let volume = ask[1].parse::<f64>().unwrap();

                book.snapshot.a.push((price, volume));
            }
        }

        if let Some(bids) = d.bids {
            for bid in bids {
                let price = bid[0].parse::<f64>().unwrap();
                let volume = bid[1].parse::<f64>().unwrap();

                book.snapshot.b.push((price, volume));
            }
        }

        if let Some(price_str) = d.price {
            let price = price_str.parse::<f64>().unwrap();

            book.snapshot.last_price = price;
        }
    }

    book.snapshot.a.sort_by(|x, y| y.0.total_cmp(&x.0));
    book.snapshot.b.sort_by(|x, y| y.0.total_cmp(&x.0));

    let mut book_lock = local_book.write().await;
    *book_lock = book;
}

async fn get_api_key() -> Result<String> {
    let url = "https://api.kucoin.com/api/v1/bullet-public";
    
    let client = reqwest::Client::new();
    let response = client.post(url)  
        .send()
        .await?;

    let data = response.json::<ApiKeyResponse>().await?;
    let api_key = data.data.token;

    println!("[KuCoin] Api Key успешно получен.");

    Ok(api_key)
}