use std::sync::Arc;

use futures_util::{StreamExt, stream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{TickerEvent, TickerInfo}, orderbook::{BookEvent, OrderBookEvent, OrderBookFromHttp, Snapshot}, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::{ExchangeStoreCMD, parse_levels__}}};


pub struct GateAdapter;

#[async_trait::async_trait]
impl ExchangeAdapter for GateAdapter {
    fn ws_url(self: Arc<Self>) -> &'static str {
        "wss://api.gateio.ws/ws/v4/"
    }

    fn requires_auth(
        self: Arc<Self>
    ) -> bool {
        false
    }

    async fn auth_url(
        self: Arc<Self>,
        _client: &reqwest::Client
    ) -> Option<url::Url> {
        todo!()
    }

    async fn get_api_key(
        self: Arc<Self>,
        _client: &reqwest::Client
    ) -> Result<String, reqwest::Error> {
        todo!()
    }

    async fn get_tickers(self: Arc<Self>, client: &reqwest::Client) -> Option<Vec<TickerInfo>> {
        let url = "https://api.gateio.ws/api/v4/spot/currency_pairs?limit=10";
        let response = client
            .get(url)
            .send()
            .await;

        let Ok(response) = response else { return None };
        
        let Ok(tickers) = response.json::<Vec<TickerInfo>>().await else { return None };
        let usdt_tickers: Vec<TickerInfo> = tickers
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn get_snapshot_spot_http(
        self: Arc<Self>,
        symbol: Symbol,
        client: &reqwest::Client,
        sender_data: mpsc::Sender<ExchangeStoreCMD>,
    ) {
        // Загружать лениво, только когда юзер просит(один раз)
        // Избегают повторную загрузку во время двух запросов одновременно для даного тикера

        let symbols = vec!["BTC_USDT", "ETH_USDT", "SOL_USDT", "BNB_USDT", "TONCOIN_USDT", "AVAX_USDT"];
        let mut urls = Vec::new();

        for symbol1 in symbols {
            let url = format!("https://api.gateio.ws/api/v4/spot/order_book?currency_pair={symbol1}&limit=1000");
            urls.push(url);
        }

        let responses = stream::iter(urls)
            .map(|url| {
                let client = client.clone();
                async move {
                    client.get(url).send().await
                }
            })
            .buffer_unordered(5);

        responses.for_each(|resp| async {
            match resp {
                Ok(response) => {
                    println!("Request done")
                },
                Err(e) => tracing::error!("{e}")
            }
        }).await;
        
        // let url = format!("https://api.gateio.ws/api/v4/spot/order_book?currency_pair={symbol}&limit=1000");
        // let response = client   
        //     .get(url)
        //     .send()
        //     .await;

        // // Доработать

        // if let Ok(response) = response {
        //     if let Ok(snapshot) = response.json::<OrderBookFromHttp>().await {
        //         let asks = parse_levels__(snapshot.asks);
        //         let bids = parse_levels__(snapshot.bids);

        //         sender_data.send(ExchangeStoreCMD::Event(
        //             BookEvent::Snapshot { 
        //                 symbol,
        //                 snapshot: Snapshot { 
        //                     a: asks, 
        //                     b: bids, 
        //                     last_update_id: None,
        //                     timestamp: 0
        //                 }
        //             }
        //         )).await.ok();
        //     }
        // }
    }

    fn create_subscribe_messages(
        self: Arc<Self>,
        symbol: Arc<Symbol>
    ) -> Vec<Message> {
        let message1 = Message::Text(
            serde_json::json!({
                "channel": "spot.order_book",
                "event": "subscribe",
                "payload": [symbol, "50", "100ms"]
            }).to_string().into()
        );

        let message2 = Message::Text(
            serde_json::json!({
                "channel": "spot.tickers",
                "event": "subscribe",
                "payload": [symbol]
            }).to_string().into()
        );

        vec![message1, message2]
    }

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        sender_data: mpsc::Sender<ExchangeStoreCMD>
    ) {
        if !msg.contains("update") {
            return;
        }
        
        if msg.contains("spot.order_book") {
            let json: OrderBookEvent = serde_json::from_str(&msg).unwrap();
            let data = json.data;
            let ts = json.timestamp;

            if let Some(data) = data {
                let ticker = data.symbol;
                if let Some(symbol) = ticker {
                    let symbol = symbol.replace("_", "").to_lowercase();

                    let asks = data.asks;
                    let bids = data.bids;

                    if let (Some(asks), Some(bids), Some(timestamp)) = (asks, bids, ts) {
                        let asks = parse_levels__(asks);
                        let bids = parse_levels__(bids);
                        
                        sender_data.send(ExchangeStoreCMD::Event(
                            BookEvent::Snapshot { 
                                symbol,
                                snapshot: Snapshot { 
                                    a: asks, 
                                    b: bids, 
                                    last_update_id: None,
                                    timestamp,
                                }
                            }
                        )).await.ok();
                    }
                }
            }
        }

        if msg.contains("spot.tickers") {
            let json: TickerEvent = serde_json::from_str(&msg).unwrap();
            if let Some(result) = json.result {
                let ticker = result.symbol;
                if let Some(symbol) = ticker {
                    let symbol = symbol.replace("_", "").to_lowercase();
                    let last_price = result.last_price;
                    let volume = result.volume;

                    if let (Some(price_str), Some(vol_str)) = (last_price, volume) {
                        let price = price_str.parse::<f64>().expect("GateAdapter -> Не удалось преобразовать last_price в f64");
                        let volume = vol_str.parse::<f64>().expect("GateAdapter -> Не удалось преобразовать volume в f64");

                        sender_data.send(ExchangeStoreCMD::Event(
                            BookEvent::TickerUpdate { 
                                symbol, 
                                last_price: price, 
                                volume: volume
                            }
                        )).await.ok();
                    }
                }
            }
        }
    }
}