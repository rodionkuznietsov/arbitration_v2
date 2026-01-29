use std::sync::Arc;

use tokio::sync::mpsc;

use crate::{exchanges::{binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, kucoin_ws::KuCoinWebsocket, mexc_ws::MexcWebsocket, orderbook::{OrderType, SnapshotUi}, websocket::Websocket}, websocket::ConnectedClient};

#[derive(Debug, Clone, PartialEq)]
pub enum ExchangeType {
    Binance, 
    Bybit,
    KuCoin,
    BinX,
    Mexc,
    Unknown
}

enum ExchangeWs {
    Bybit(Arc<BybitWebsocket>),
    KuCoin(Arc<KuCoinWebsocket>),
    BinX(Arc<BinXWebsocket>),
    Mexc(Arc<MexcWebsocket>)
}

impl ExchangeWs {
    fn ticker_tx(&self) -> async_channel::Sender<(String, String)> {
        match self {
            Self::Bybit(ws) => {
                ws.ticker_tx.clone()
            }
            Self::KuCoin(ws) => {
                ws.ticker_tx.clone()
            }
            Self::BinX(ws) => {
                ws.ticker_tx.clone()
            }
            Self::Mexc(ws) => {
                ws.ticker_tx.clone()
            }
        }
    }

    async fn get_snapshot(&self, snapshot_tx: mpsc::UnboundedSender<SnapshotUi> ) {
        match self {
            Self::Bybit(ws) => {
                ws.clone().get_snapshot(snapshot_tx).await
            }
            Self::KuCoin(ws) => {
                ws.clone().get_snapshot(snapshot_tx).await
            }
            Self::BinX(ws) => {
                ws.clone().get_snapshot(snapshot_tx).await
            }
            Self::Mexc(ws) => {
                ws.clone().get_snapshot(snapshot_tx).await
            }
        }
    }
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
) {

    let kucoin_websocket = KuCoinWebsocket::new(true);
    let bybit_websocket = BybitWebsocket::new(true);
    let binx_websocket = BinXWebsocket::new(true);
    let mexc_websocket = MexcWebsocket::new(false);
    
    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let client = client.clone();

        tokio::spawn({
            // let binance_book = binance_book.clone();
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();

            let token = token.clone();
            let client = client.clone();

            async move {
                if long_exchange != ExchangeType::Unknown {
                    let websocket = match long_exchange {
                        ExchangeType::Binance => return,
                        ExchangeType::Bybit => ExchangeWs::Bybit(bybit),
                        ExchangeType::KuCoin => ExchangeWs::KuCoin(kucoin),
                        ExchangeType::BinX => ExchangeWs::BinX(binx),
                        ExchangeType::Mexc => ExchangeWs::Mexc(mexc),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel();
                    
                    tokio::spawn(async move {
                        websocket.get_snapshot(snapshot_tx).await;
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      

                        tokio::select! {
                            _ = token.cancelled() => return ,
                            _ = client.send_snapshot(OrderType::Long, snapshot) => {}
                        }
                    }
                }
            }
        });

        tokio::spawn({
            // let binance_book = binance_book.clone();
            let bybit = bybit_websocket.clone();
            let kucoin = kucoin_websocket.clone();
            let binx = binx_websocket.clone();
            let mexc = mexc_websocket.clone();
            
            let token = token.clone();
            let client = client.clone();

            async move {
                if short_exchange != ExchangeType::Unknown {
                    let websocket = match short_exchange {
                        ExchangeType::Binance => return,
                        ExchangeType::Bybit => ExchangeWs::Bybit(bybit),
                        ExchangeType::KuCoin => ExchangeWs::KuCoin(kucoin),
                        ExchangeType::BinX => ExchangeWs::BinX(binx),
                        ExchangeType::Mexc => ExchangeWs::Mexc(mexc),
                        ExchangeType::Unknown => return,
                    };

                    websocket.ticker_tx().send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel();
                    
                    tokio::spawn(async move {
                        websocket.get_snapshot(snapshot_tx).await;
                    });
                    
                    while let Some(snapshot) = snapshot_rx.recv().await {
                        let mut client = client.clone();      

                        tokio::select! {
                            _ = token.cancelled() => return ,
                            _ = client.send_snapshot(OrderType::Short, snapshot) => {}
                        }
                    }
                }
            }
        });
    }
}
