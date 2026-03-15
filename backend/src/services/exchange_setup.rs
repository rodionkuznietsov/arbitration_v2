use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};
use crate::exchanges::exchange_adapter::ExchangeAdapter;
use crate::models::exchange::TickerInfo;
use crate::models::websocket::{WsCmd};
use crate::models::{exchange::ExchangeType, orderbook::SnapshotUi};
use crate::services::data_aggregator::DataAggregatorCmd;
use crate::services::{exchange_aggregator::{ExchangeStore, ExchangeStoreCMD}};

const CHUNK_SIZE: usize = 50;

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn spawn_quote_updater(self: Arc<Self>);
    fn spawn_volume_updater(self: Arc<Self>);
    fn spawn_oderbooks_updater(self: Arc<Self>);
}

/// Init Exchange
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
    pub books_updates: broadcast::Sender<String>,
    exchange_id: ExchangeType,

    data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>
}

impl<A: ExchangeAdapter + Send + Sync + 'static> ExchangeSetup<A> {
    pub fn new(
        exchange_id: ExchangeType,
        adapter: Arc<A>,
        enabled: bool,
        data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>
    ) -> Arc<Self> {
        let title = format!("{}Websocket", exchange_id);
        let (ticker_tx, ticker_rx) = async_channel::bounded(1);
        let client = reqwest::Client::new();
        let (sender_data, rx_data) = mpsc::channel::<ExchangeStoreCMD>(1);

        let store = ExchangeStore::new(rx_data);
        let books_updates = store.book_updates.clone();
        tokio::spawn(async move {
            store.set_data().await;
        });

        let this = Arc::new(Self {
            title, enabled,
            ticker_tx, ticker_rx, client,
            sender_data, books_updates,
            exchange_id, data_aggregator_tx, adapter
        });

        this.clone().spawn_quote_updater();
        this.clone().spawn_volume_updater();
        this.clone().spawn_oderbooks_updater();

        this
    }

    pub fn start(self: Arc<Self>) {
        if !self.enabled {
            tracing::warn!("{} disabled", self.title);
            return;
        }
        
        tokio::spawn({
            async move {
                let tickers = self.adapter.clone().get_tickers(&self.client).await;
                if let Some(result) = tickers {
                    self.try_run_ws_session(&result).await;
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

#[async_trait]
impl<A: ExchangeAdapter + Send + Sync + 'static> ExchangeWebsocket for ExchangeSetup<A> {
    fn spawn_quote_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        let title = self.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(symbol) => {
                        let symbol = Arc::new(symbol);
                        let (reply, rx) = oneshot::channel::<(f64, f64)>();
                        self.sender_data.send(ExchangeStoreCMD::GetQuote { symbol: symbol.clone(), reply: reply }).await.ok();

                        if let Ok((ask, bid)) = rx.await {
                            self.data_aggregator_tx.clone().send(
                                DataAggregatorCmd::UpdateQuotes { 
                                    exchange_id: self.exchange_id,
                                    symbol: symbol.clone(),
                                    ask,
                                    bid
                                }
                            ).await.ok();
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал спреда закрыт", title);
                        break;
                    }
                }
            }
        });
    }

    fn spawn_volume_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        let title = self.title.clone();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(symbol) => {
                        let symbol = Arc::new(symbol);
                        let (reply, rx) = oneshot::channel::<f64>();
                        if self.sender_data.send(ExchangeStoreCMD::GetVolume { symbol: symbol.clone(), reply }).await.is_err() {
                            continue;
                        }

                        if let Ok(volume) = rx.await {                            
                            self.data_aggregator_tx.clone().send(
                                DataAggregatorCmd::UpdateVolumes { 
                                    exchange_id: self.exchange_id, 
                                    volume, 
                                    symbol: symbol
                                }
                            ).await.ok();
                        }
                    },
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал volume закрыт", title);
                        break;
                    }
                }
            }
        });
    }

    fn spawn_oderbooks_updater(
        self: Arc<Self>,
    ) {
        let mut rx = self.books_updates.subscribe();
        
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(symbol) => {
                        let symbol = Arc::new(symbol);
                        let (reply, rx) = oneshot::channel::<SnapshotUi>();
                        self.sender_data.send(ExchangeStoreCMD::GetBook { symbol: symbol.clone(), reply }).await.ok();

                        if let Ok(snapshot) = rx.await {
                            self.data_aggregator_tx.send(
                                DataAggregatorCmd::UpdateOrderbooks { 
                                    exchange_id: self.exchange_id,
                                    snapshot_ui: snapshot,
                                    symbol,
                                    _created_at: Instant::now()
                                }
                            ).await.ok();
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("{} Канал orderbook закрыт", self.title);
                        break;
                    }
                    _ => continue
                }
            }
        });
    }
}
