use std::sync::Arc;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info};
use crate::models::exchange::TickerInfo;
use crate::models::websocket::{WsCmd};
use crate::models::{exchange::ExchangeType};
use crate::services::data_aggregator::DataAggregatorCmd;
use crate::services::exchange::exchange_adapter::ExchangeAdapter;
use crate::services::exchange::exchange_aggregator::{ExchangeStore, ExchangeStoreCMD};
use crate::services::exchange::exchange_channel_store::ExchangeChannelStoreCmd;

const CHUNK_SIZE: usize = 50;

/// <b>ExchangeSetup</b> инициализирует WebSocket с помощью `ExchangeAdapter`
pub struct ExchangeSetup<T: ExchangeAdapter> {
    pub adapter: Arc<T>,
    pub title: String,
    pub enabled: bool,
    #[allow(unused)]
    pub ticker_tx: async_channel::Sender<(String, String)>,
    #[allow(unused)]
    pub ticker_rx: async_channel::Receiver<(String, String)>,
    pub client: reqwest::Client,
    pub sender_data: mpsc::Sender<ExchangeStoreCMD>,
    exchange_id: ExchangeType,

    data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>
}

impl<A: ExchangeAdapter + Send + Sync + 'static> ExchangeSetup<A> {
    pub fn new(
        exchange_id: ExchangeType,
        adapter: Arc<A>,
        enabled: bool,
        data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>,
        exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>
    ) -> Arc<Self> {
        let title = format!("{}Websocket", exchange_id);
        let (ticker_tx, ticker_rx) = async_channel::bounded(64);
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<ExchangeStoreCMD>(64);

        let store = ExchangeStore::new(rx_data, exchange_id);
        let sender_data_cl = sender_data.clone();

        tokio::spawn(async move {
            exchange_channel_store_tx.send_timeout(
                ExchangeChannelStoreCmd::RegisterChannel { 
                    exchange_id, 
                    channel: sender_data_cl
                }, 
                Duration::from_millis(10)
            ).await.ok();
            
            store.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled,
            ticker_tx, ticker_rx, client,
            sender_data,
            exchange_id, data_aggregator_tx, adapter
        });

        this
    }

    pub fn start(self: Arc<Self>) {
        if !self.enabled {
            tracing::warn!("{} disabled", self.title);
            return;
        }

        tokio::spawn({
            let this = self.clone();
            async move {
                let tickers = this.adapter.clone().get_tickers(&this.client).await;
                if let Some(result) = tickers {                    
                    // Загружаем снапшоты
                    // tokio::spawn({
                    //     let this = self.clone();
                    //     let adapter = this.adapter.clone();
                    //     async move {
                    //      adapter.get_snapshot_spot_http(&result, &this.client, this.sender_data.clone()).await;
                    //     }
                    // });

                    this.try_run_ws_session(&result).await;
                }
            }
        });
    }

    async fn try_run_ws_session(
        self: Arc<Self>,
        tickers: &Vec<TickerInfo>
    ) {
        let adapter = self.adapter.clone();

        for chunk in tickers.chunks(CHUNK_SIZE) {
            let (cmd_tx, mut cmd_rx) = mpsc::channel::<WsCmd>(CHUNK_SIZE);

            for ticker_info in chunk {
                let symbol = ticker_info.symbol.clone();
                if let Some(symbol) = symbol {
                    
                    let symbol = Arc::new(symbol);

                    // Регистрируем тикеры в exchange aggregator
                    self.sender_data.send(ExchangeStoreCMD::RegisterSymbol { symbol: symbol.clone() }).await.ok();

                    // Регистрируем тикеры с exchange_id в общем аггрегаторе
                    self.data_aggregator_tx.send(
                        DataAggregatorCmd::MarketRegister { 
                            symbol: symbol.clone(), 
                            exchange_id: self.exchange_id 
                        }
                    ).await.ok();

                    let messages = adapter.clone().create_subscribe_messages(symbol);
                    cmd_tx.send(WsCmd::Subscribe(messages)).await.ok();
                }
            }

            let this = self.clone();
            tokio::spawn(async move {
                this.connect_ws(&mut cmd_rx).await;
            });
        }
    }

    async fn connect_ws(
        self: Arc<Self>,
        cmd_rx: &mut mpsc::Receiver<WsCmd>
    ) {
        let adapter = self.adapter.clone();
        let ws_url = adapter.clone().ws_url();

        let mut ws_stream = Some(connect_async(ws_url).await.unwrap().0);
        if adapter.clone().requires_auth() {
            let url = adapter.clone().auth_url(&self.client).await;
            if let Some(ws_url) = url {
                ws_stream = Some(connect_async(ws_url).await.unwrap().0);
            }
        }

        if let Some(ws_stream) = ws_stream {
            let (mut write, mut read) = ws_stream.split();
            
            info!("{} -> is running", self.title);

            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    WsCmd::Subscribe(msgs) => {
                        for msg in msgs {
                            write.send(msg).await.ok();
                        }
                    }
                }
            }

            while let Some(result) = read.next().await {
                if let Ok(msg) = result {
                    match msg {
                        Message::Text(channel) => {
                            adapter.clone().parse_message(channel, self.sender_data.clone()).await;
                        },
                        Message::Pong(pong) => {
                            println!("{} ответил на Pong: {:?}", self.title, pong)
                        },
                        Message::Close(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn _run_ws_session(
        self: Arc<Self>,
        _tickers: &Vec<TickerInfo>
    ) {

    }
}
