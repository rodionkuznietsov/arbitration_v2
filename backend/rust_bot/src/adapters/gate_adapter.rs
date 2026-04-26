use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, Semaphore, mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{PriceCache, TickerEvent, TickerInfo}, orderbook::{BookEvent, OrderBookEvent, OrderBookEventData, Snapshot}, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::{ExchangeStoreCMD, parse_levels__}}};

pub struct GateAdapter {
    price_cache: Arc<Mutex<PriceCache>>
}

impl GateAdapter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { price_cache: Arc::new(Mutex::new(PriceCache::new())) })
    }
}

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
        tickers: &Vec<TickerInfo>,
        client: &reqwest::Client,
        sender_data: watch::Sender<ExchangeStoreCMD>,
    ) {
        // Загружать лениво, только когда юзер просит(один раз)
        // Избегают повторную загрузку во время двух запросов одновременно для даного тикера

        let mut urls = Vec::new();

        // Формируем url ссылки
        for info in tickers {
            let symbol = info.symbol.clone();
            if let Some(symbol) = symbol {
                let url = format!("https://api.gateio.ws/api/v4/spot/order_book?currency_pair={symbol}&limit=1000");
                urls.push((url, symbol));
            }
        }

        let semaphore = Arc::new(Semaphore::new(8));

        for chunk in urls[0..1].chunks(8) {
            for (url, symbol) in chunk {
                let url = url.clone();
                let symbol = symbol.clone();
                let permit = semaphore.clone().acquire_owned().await;
                let client = client.clone();
                let _sender_data = sender_data.clone();

                tokio::spawn(async move {
                    let _p = permit;
                    tracing::info!("Запрос {}", symbol);

                    let _response = client.get(url).send().await;
                    // match response {
                    //     Ok(response) => {
                    //         let data: Result<OrderBookFromHttp<'_>, reqwest::Error> = response.json().await;
                    //         if let Ok(snapshot) = data {
                    //             let asks = parse_levels__(snapshot.asks);
                    //             let bids = parse_levels__(snapshot.bids);

                    //             sender_data.send(ExchangeStoreCMD::Event(
                    //                 BookEvent::Snapshot { 
                    //                     symbol,
                    //                     snapshot: Snapshot { 
                    //                         a: asks, 
                    //                         b: bids, 
                    //                         last_update_id: None,
                    //                         timestamp: 0
                    //                     }
                    //                 }
                    //             )).ok();
                    //         }
                    //     },
                    //     Err(e) => {
                    //         tracing::error!("{e}")
                    //     }
                    // }

                    tracing::info!("Запрос завершен");
                });
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
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

    fn cache(
        &self,
    ) -> &Arc<Mutex<PriceCache>> {
        &self.price_cache
    }

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        let msg_arc = Arc::new(msg);
        if !msg_arc.contains("update") {
            return;
        }
        
        if msg_arc.contains("spot.order_book") {
            self.clone().parse_orderbook(msg_arc.clone(), snapshot_channel, sender_data.clone()).await;
        }

        if msg_arc.contains("spot.tickers") {
            self.parse_tickers(msg_arc, sender_data).await;
        }
    }

    async fn parse_tickers(
        self: Arc<Self>,
        msg: Arc<String>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        let json: TickerEvent<'_> = serde_json::from_str(&msg).unwrap();
        if let Some(result) = json.result {
            let ticker = result.symbol;
            let last_price = result.last_price;
            let volume = result.volume;

            if let (
                Some(symbol), 
                Some(price_str), 
                Some(vol_str)
            ) = (
                ticker, 
                last_price, 
                volume
            ) {
                let symbol = symbol.replace("_", "").to_lowercase();
                let last_price = price_str.parse::<f64>().expect("GateAdapter -> Не удалось преобразовать last_price в f64");
                let volume = vol_str.parse::<f64>().expect("GateAdapter -> Не удалось преобразовать volume в f64");
                
                let is_valid_price = self.is_valid_price(last_price, &symbol).await;
                if is_valid_price {
                    let _ = sender_data.send(ExchangeStoreCMD::Event(
                        BookEvent::TickerUpdate { 
                            symbol, 
                            last_price: last_price, 
                            volume: volume
                        }
                    ));
                }
            }
        }
    }

    async fn parse_orderbook(
        self: Arc<Self>,
        msg: Arc<String>,
        snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        let json: OrderBookEvent<'_> = serde_json::from_str(&msg).unwrap();
        let data = json.data;
        self.handle_snapshot(data, snapshot_channel, sender_data.clone()).await;
    }

    async fn handle_snapshot<'a>(
        self: Arc<Self>,
        data: Option<OrderBookEventData<'a>>,
        _snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        if let Some(data) = data {
            let ticker = data.symbol;
            if let Some(symbol) = ticker {
                let symbol = symbol.replace("_", "").to_lowercase();

                let asks = data.asks;
                let bids = data.bids;

                if let (
                    Some(asks), 
                    Some(bids)
                ) = (asks, bids) {
                    let asks = parse_levels__(asks);
                    let bids = parse_levels__(bids);
                    
                    let is_valid_book = self.is_valid_book(&asks, &bids);
                    if is_valid_book {
                        let _ = sender_data.send(ExchangeStoreCMD::Event(
                            BookEvent::Snapshot { 
                                symbol,
                                snapshot: Snapshot { 
                                    a: asks, 
                                    b: bids, 
                                    last_update_id: None,
                                }
                            },
                        ));
                    }
                }
            }
        }
    }

    async fn handle_delta<'a>(
        self: Arc<Self>,
        _data: Option<OrderBookEventData<'a>>,
        _sender_data: watch::Sender<ExchangeStoreCMD>
    ) {

    }
}