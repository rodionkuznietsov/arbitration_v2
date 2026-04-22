use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{models::{exchange::{TickerInfo, TickerResponse}, exchange_key::ApiKeyResponse, websocket::Symbol}, services::exchange::{exchange_adapter::ExchangeAdapter, exchange_aggregator::ExchangeStoreCMD}};

pub struct KuCoinAdapter;

#[async_trait::async_trait]
impl ExchangeAdapter for KuCoinAdapter {
    fn ws_url(
        self: Arc<Self>
    ) -> &'static str {
        "wss://ws-api-spot.kucoin.com"
    }

    fn requires_auth(
        self: Arc<Self>
    ) -> bool {
        true
    }

    async fn auth_url(
        self: Arc<Self>,
        client: &reqwest::Client
    ) -> Option<url::Url> {
        let key = self.clone().get_api_key(client).await;
        if let Ok(token) = key {
            let url = format!("{}?token={}", self.ws_url(), token);
            let ws_url = url::Url::parse(&url).unwrap();
            return Some(ws_url);
        } else {
            return None;
        }
    }

    async fn get_api_key(
        self: Arc<Self>,
        client: &reqwest::Client
    ) -> Result<String, reqwest::Error> {
        let url = "https://api.kucoin.com/api/v1/bullet-public";
        
        let response = client.post(url)  
            .send()
            .await?;

        let data = response.json::<ApiKeyResponse>().await?;
        let api_key = data.data.token;

        println!("[KuCoin-Rest] Api-Key успешно получен.");

        Ok(api_key)
    }

    async fn get_tickers(
        self: Arc<Self>, 
        client: &reqwest::Client
    ) -> Option<Vec<TickerInfo>> {
        let url = "https://api.kucoin.com/api/v1/market/allTickers";
        let response = client.get(url).send().await;
        
        let Ok(response) = response else { return None };
        let Ok(json) = response.json::<TickerResponse>().await else { return None };

        let usdt_tickers: Vec<TickerInfo> = json.result.list
            .into_iter()
            .filter(|x| x.symbol.clone().unwrap().ends_with("-USDT"))
            .collect();

        Some(usdt_tickers)
    }

    async fn get_snapshot_spot_http(
        self: Arc<Self>,
        tickers: &Vec<TickerInfo>,
        client: &reqwest::Client,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {
        
    }

    fn create_subscribe_messages(
        self: Arc<Self>,
        symbol: Arc<Symbol>
    ) -> Vec<Message> {
        let message1 = Message::Text(
            serde_json::json!({
                "type": "subscribe",
                "topic": format!("/spotMarket/level2Depth50:{}", symbol),
                "response": true,
                "privateChannel": false,
            }).to_string().into()
        );

        let message2 = Message::Text(
            serde_json::json!({
                "type": "subscribe",
                "topic": format!("/market/ticker:{}", symbol),
                "response": true,
                "privateChannel": false,
            }).to_string().into()
        );

        let message3 = Message::Text(
            serde_json::json!({
                "type": "subscribe",
                "topic": format!("/market/snapshot:{}", symbol),
                "response": true,
                "privateChannel": false,
            }).to_string().into()
        );

        vec![message1, message2, message3]
    }

    async fn parse_message(
        self: Arc<Self>,
        msg: String,
        _snapshot_channel: mpsc::Sender<ExchangeStoreCMD>,
        _data_aggregator_tx: watch::Sender<ExchangeStoreCMD>
    ) {
        // Парсим orderbook
        if msg.contains("level2Depth50") {

        }

        // Парсим price
        if msg.contains("trade.ticker") {

        }
    }

    async fn parse_tickers(
        self: Arc<Self>,
        msg: Arc<String>,
        sender_data: watch::Sender<ExchangeStoreCMD>
    ) {

    }

    fn parse_orderbook(
        self: Arc<Self>,
        msg: Arc<String>
    ) {
        
    }
    
}