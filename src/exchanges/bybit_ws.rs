use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

use crate::exchanges::{orderbook::{LocalOrderBook, Snapshot}, websocket::{Ticker, Websocket}};

#[derive(Deserialize, Debug, Serialize)]
struct TickerResponse {
    #[serde(rename="result")]
    result: TickerResult
}

#[derive(Deserialize, Debug, Serialize)]
struct TickerResult {
    #[serde(rename="list")]
    list: Vec<Ticker>
}

// #[derive(Deserialize, Debug, Serialize)]
// struct Ticker {
//     #[serde(rename="symbol")]
//     symbol: String
// }

#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookEvent {
    #[serde(rename="type")]
    order_type: Option<String>,
    #[serde(rename="data")]
    data: Option<OrderBookEventData>
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct OrderBookEventData {
    #[serde(rename="s")]
    symbol: Option<String>,
    #[serde(rename="a")]
    asks: Option<Vec<Vec<String>>>,
    #[serde(rename="b")]
    bids: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerEvent {
    #[serde(rename="data")]
    data: Option<TickerEventData>
}

#[derive(Debug, Deserialize, Serialize)]
struct TickerEventData {
    #[serde(rename="symbol")]
    symbol: String,
    #[serde(rename="lastPrice")]
    last_price: String
}

#[derive(Clone)]
pub struct BybitWebsocket {
    title: String,
    enabled: bool,
    local_book: Arc<RwLock<LocalOrderBook>>,
}

impl BybitWebsocket {
    pub fn new(enabled: bool) -> Arc<Self> {
        let local_book = LocalOrderBook::new();
        let title = "[Bybit-Websocket]".to_string();
        let this = Arc::new(Self { enabled, local_book, title });

        let this_cl = Arc::clone(&this);
        this_cl.connect("spot".to_string());
        this
    }
}

impl Websocket for BybitWebsocket {
    type Snapshot = OrderBookEvent;
    type Price = TickerEvent;

    fn connect(self: Arc<Self>, channel_type: String) {
        if !self.enabled {
            println!("{} enabled: false", self.title);
            return;
        } 

        // let this = Arc::clone(&self);
        // tokio::spawn({
        //     async move {
        //         let tickers = this.get_tickers(&channel_type).await.expect(&format!("{} cannot load tokens", this.title));

        //         let url = url::Url::parse("wss://stream.bybit.com/v5/public/spot").unwrap();
        //         let (ws_stream, _) = connect_async(url.to_string()).await.expect("[Bybit] Failed to connect");
        //         let (mut write, read) = ws_stream.split();

        //         println!("🌐 {} is running", this.title);

        //         let mut params = Vec::new();

        //         for ticker in tickers {
        //             let symbol = ticker.symbol;
        //             let orderbook_str = format!("orderbook.200.{}", symbol.clone().unwrap().to_uppercase());
        //             let price_str = format!("tickers.{}", symbol.unwrap().to_uppercase());

        //             params.push(orderbook_str);
        //             params.push(price_str);
        //         }

        //         for chunk in params.chunks(10) {
        //             write.send(Message::Text(
        //                 serde_json::json!({
        //                     "op": "subscribe",
        //                     "channel_type": channel_type,
        //                     "args": chunk
        //                 }).to_string().into()
        //             )).await.unwrap();
        //         }

        //         let (tx_event, mut rx_event) = mpsc::channel::<TickerEvent>(1000);

        //         let this_cl = this.clone();
        //         tokio::spawn(async move {
        //             while let Some(event) = rx_event.recv().await {
        //                 this_cl.clone().handle_price(event).await;
        //             }
        //         });

        //         let read_future = read.for_each(|msg| async {
        //             let msg_type = msg.unwrap();

        //             match msg_type {
        //                 Message::Text(channel) => {
        //                     if channel.contains("orderbook.") {
        //                         let json = serde_json::from_str::<OrderBookEvent>(&channel).unwrap();
        //                         match json.order_type.as_deref() {
        //                             Some("snapshot") => this.clone().handle_snapshot(json).await.unwrap(),
        //                             Some("delta") => this.clone().handle_delta(json).await,
        //                             _ => {}
        //                         }
        //                     }
                            
        //                     if channel.contains("tickers.") {
        //                         let json = serde_json::from_str::<TickerEvent>(&channel).unwrap();
        //                         tx_event.send(json).await.unwrap();
        //                     }
        //                 }
        //                 _ => { 
        //                     eprintln!("[Bybit-Websocket]: another type")
        //                 }
        //             }
                    
        //         });
        //         read_future.await;
        //     }
        // });
    }

    fn get_local_book(self: Arc<Self>,) -> Arc<RwLock<LocalOrderBook>> {
        self.local_book.clone()
    }

    async fn get_tickers(&self, channel_type: &str) -> Result<Vec<Ticker>> {
        let url = format!("https://api.bybit.com/v5/market/tickers?category={channel_type}");
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await?;

        let mut usdt_tickers = Vec::new();

        if response.status().is_success() {
            let json = response.json::<TickerResponse>().await?;
            usdt_tickers = json.result.list
                .into_iter()
                .filter(|ticker| ticker.symbol.clone().unwrap().ends_with("USDT"))
                .collect();
        }

        Ok(usdt_tickers)
    }

    async fn handle_snapshot(self: Arc<Self>, json: OrderBookEvent) -> Option<(String, BTreeMap<i64, f64>, BTreeMap<i64, f64>)> {
        let book = {
            self.local_book.read().await.clone()
        };

        match json.data {
            Some(data) => {
                let ticker = data.symbol.unwrap().to_lowercase();

                // println!("{:?}", ticker);

                let asks = book.parse_levels(data.asks.unwrap()).await;
                let bids = book.parse_levels(data.bids.unwrap()).await;

                // Добавляем Asks/Bids в локальный orderbook
                book.books.insert(ticker, Snapshot {
                    a: asks,
                    b: bids,
                    last_price: 0.0,
                });
            }
            _ => {}
        }

        let mut lock = self.local_book.write().await;
        *lock = book;

        Some((String::new(), BTreeMap::new(), BTreeMap::new()))
    }

    async fn handle_delta(self: Arc<Self>, json: OrderBookEvent) {
        let mut lock = self.local_book.write().await;

        match json.data {
            Some(data) => {
                let ticker = data.symbol.unwrap().to_lowercase();
                let asks = lock.parse_levels(data.asks.unwrap()).await;
                let bids = lock.parse_levels(data.bids.unwrap()).await;

                lock.apply_snapshot_updates(&ticker, asks, bids).await;
            }
            _ => {}
        }
        drop(lock);
    }

    async fn handle_price(self: Arc<Self>, json: TickerEvent) -> Option<(String, f64)> {
        let mut lock = self.local_book.write().await;
        
        if let Some(data) = json.data {
            let ticker = data.symbol.to_lowercase();
            let last_price = data.last_price.parse::<f64>().expect("[Bybit-Websocket] bad price");

            lock.set_last_price(&ticker, last_price).await;
        }
        drop(lock);

        Some((String::new(), 0.0))
    }
}
