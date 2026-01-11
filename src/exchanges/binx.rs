use std::{io::Read, sync::Arc};

use flate2::read::MultiGzDecoder;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::exchanges::orderbook::LocalOrderBook;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Snapshot {
    #[serde(rename="data")]
    data: Option<SnapshotData>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SnapshotData {
    #[serde(rename="asks")]
    asks: Option<Vec<Vec<String>>>,
    #[serde(rename="bids")]
    bids: Option<Vec<Vec<String>>>,
    #[serde(rename="c")]
    price: Option<String>
}

pub async fn connect(ticker: &str, _channel_type: &str, local_book: Arc<RwLock<LocalOrderBook>>) {
    let url = url::Url::parse("wss://open-api-ws.bingx.com/market").unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.unwrap();
    let (mut write, read) = ws_stream.split();

    println!("🌐 [BinX-Websocket] is running");

    let ticker = {
        if ticker.to_lowercase() == "ton" {
            "toncoin".to_string()
        } else {
            ticker.to_string()
        }
    };

    let orderbook = format!("{}-USDT@depth50", ticker.to_uppercase());

    write.send(Message::Text(
        serde_json::json!({
            "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
            "reqType": "sub",
            "dataType": orderbook
        }).to_string().into()
    )).await.unwrap();
    
    let price = format!("{}-USDT@lastPrice", ticker.to_uppercase());
    write.send(Message::Text(
        serde_json::json!({
            "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
            "reqType": "sub",
            "dataType": price
        }).to_string().into()
    )).await.unwrap();

    let read_future = read.for_each(|msg| async {
        let data = msg.unwrap();
        let book = local_book.clone();
        match data {
            Message::Text(_) => {}
            Message::Binary(binary) => {
                let mut d = MultiGzDecoder::new(&*binary);
                let mut s = String::new();
                d.read_to_string(&mut s).unwrap();

                let data = serde_json::from_str::<Snapshot>(&s).unwrap();
                
                fetch_data(data.clone(), book.clone()).await;
            }
            _ => {}
        }
    });

    read_future.await;
}

async fn fetch_data(data: Snapshot, local_book: Arc<RwLock<LocalOrderBook>>) {
    let mut book = {
        let current = local_book.read().await;
        current.clone()
    };
    
    if let Some(data) = data.data {

        if let Some(price_str) = data.price {
            let price = price_str.parse::<f64>().unwrap_or(0.0);
            book.snapshot.last_price = price
        }

        if let Some(asks_array) = data.asks {
            for ask in asks_array {
                let price = ask[0].parse::<f64>().unwrap_or(0.0);
                let volume = ask[1].parse::<f64>().unwrap_or(0.0);

                if let Some(x) = book.snapshot.a.iter_mut().find(|x| x.0 == price) {
                    x.1 = volume
                } else {
                    book.snapshot.a.push((price, volume));
                }
            }
        }
        
        if let Some(bids_array) = data.bids {
            for bid in bids_array {
                let price = bid[0].parse::<f64>().unwrap_or(0.0);
                let volume = bid[1].parse::<f64>().unwrap_or(0.0);

                if let Some(x) = book.snapshot.b.iter_mut().find(|x| x.0 == price) {
                    x.1 = volume
                } else {
                    book.snapshot.b.push((price, volume));
                }
            }
        }
    }

    // Меняем уровень ask

    // Сортирируем от большого к низкому
    book.snapshot.a.sort_by(|x, y| x.0.total_cmp(&y.0));

    // Определяем стартовую позицию
    let astart = match book.snapshot.a.binary_search_by(|x| {
        if x.0 >= book.snapshot.last_price && book.snapshot.last_price > 0.0 { 
            std::cmp::Ordering::Greater
         }
        else { std::cmp::Ordering::Less }
    }) {
        Ok(pos) | Err(pos) => pos
    };

    book.snapshot.a = book.snapshot.a[astart..].to_vec();

    // Теперь сортирируем в нужном порядке с большего к меньшему
    book.snapshot.a.sort_by(|x, y| y.0.total_cmp(&x.0));

    
    // Меняем уровень bids
    book.snapshot.b.sort_by(|x, y| y.0.total_cmp(&x.0));
    let bstart = match book.snapshot.b.binary_search_by(|x| {
        if x.0 <= book.snapshot.last_price && book.snapshot.last_price > 0.0 { 
            std::cmp::Ordering::Greater
         }
        else { std::cmp::Ordering::Less }
    }) {
        Ok(pos) | Err(pos) => pos
    };

    book.snapshot.b = book.snapshot.b[bstart..].to_vec();

    let mut book_lock = local_book.write().await;
    *book_lock = book;
}
