use std::{sync::Arc};
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{exchange::ExchangeType, orderbook::{OrderType, SnapshotUi}, websocket::{ChannelType, ServerToClientEvent}}, storage::{candle::get_user_candles}, transport::ws::ConnectedClient};

#[async_trait]
pub trait ExchangeWebsocket: Send + Sync {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)>;
    async fn get_snapshot(self: Arc<Self>, snapshot_tx: mpsc::Sender<SnapshotUi>); 
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
    pool: sqlx::PgPool
) {

    let kucoin_websocket = KuCoinWebsocket::new(false);
    let bybit_websocket = BybitWebsocket::new(true);
    let binx_websocket = BinXWebsocket::new(false);
    let mexc_websocket = MexcWebsocket::new(false);
    let binance_websocket = BinanceWebsocket::new(false);
    let gate_websocket = GateWebsocket::new(true);
    let lbank_websocket = LBankWebsocket::new(false);
    
    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let exchange_pair = client.exchange_pair.clone();
        let ticker = client.ticker.clone();
        let client = client.clone();

        // Инициализация последних 100 свечей
        tokio::spawn({
            let pool = pool.clone();
            let mut client = client.clone();
            async move {
                if !exchange_pair.is_empty() {
                    let init_candles = get_user_candles(&pool, &ticker, &exchange_pair).await;
                    if let Ok(candles) = init_candles {
                        client.send_to_client(ServerToClientEvent::CandlesHistory(ChannelType::CandlesHistory, candles, ticker)).await;
                    }
                }
            }
        });

        tokio::spawn({
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();
            let binance = binance_websocket.clone();
            let gate = gate_websocket.clone();
            let lbank = lbank_websocket.clone();

            let token = token.clone();
            let client = client.clone();

            async move {
                if long_exchange != ExchangeType::Unknown {
                    let websocket: Arc<dyn ExchangeWebsocket> = match long_exchange {
                        ExchangeType::Binance => binance.clone(),
                        ExchangeType::Bybit => bybit.clone(),
                        ExchangeType::KuCoin => kucoin.clone(),
                        ExchangeType::BinX => binx.clone(),
                        ExchangeType::Mexc => mexc.clone(),
                        ExchangeType::Gate => gate.clone(),
                        ExchangeType::LBank => lbank.clone(),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::channel(100);
                    
                    tokio::spawn({
                        let token  = token.clone();
                        async move {
                            tokio::select! {
                                _ = token.cancelled() => return,
                                _ = websocket.get_snapshot(snapshot_tx) => {}
                            }
                        }
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      
                        let ticker = client.ticker.clone();

                        tokio::select! {
                            _ = token.cancelled() => return,
                            _ = client.send_to_client(
                                    ServerToClientEvent::OrderBook(
                                    ChannelType::OrderBook, 
                                    OrderType::Long, 
                                    snapshot, 
                                    ticker
                                )
                            ) => {}
                        }
                    }

                    return;
                }
            }
        });

        tokio::spawn({
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();
            let binance = binance_websocket.clone();
            let gate = gate_websocket.clone();
            let lbank = lbank_websocket.clone();
            
            let token = token.clone();
            let client = client.clone();

            async move {
                if short_exchange != ExchangeType::Unknown {
                    let websocket: Arc<dyn ExchangeWebsocket> = match short_exchange {
                        ExchangeType::Binance => binance.clone(),
                        ExchangeType::Bybit => bybit.clone(),
                        ExchangeType::KuCoin => kucoin.clone(),
                        ExchangeType::BinX => binx.clone(),
                        ExchangeType::Mexc => mexc.clone(),
                        ExchangeType::Gate => gate.clone(),
                        ExchangeType::LBank => lbank.clone(),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::channel(100);
                    
                    tokio::spawn({
                        let token  = token.clone();
                        async move {
                            tokio::select! {
                                _ = token.cancelled() => return,
                                _ = websocket.get_snapshot(snapshot_tx) => {}
                            }
                        }
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      
                        let ticker = client.ticker.clone();

                        tokio::select! {
                            _ = token.cancelled() => return ,
                            _ = client.send_to_client(
                                    ServerToClientEvent::OrderBook(
                                    ChannelType::OrderBook, 
                                    OrderType::Short, 
                                    snapshot, 
                                    ticker
                                )
                            ) => {}
                        }
                    }

                    return ;
                }
            }
        });
    }
}