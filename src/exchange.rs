use std::{collections::HashMap, time::Duration};

use tokio::sync::mpsc;

use crate::{exchanges::{bybit_ws::BybitWebsocket, kucoin_ws::KuCoinWebsocket, orderbook::{LocalOrderBook, OrderBookManager, OrderType}, websocket::Websocket}, websocket::ConnectedClient};

#[derive(Debug, Clone, PartialEq)]
pub enum ExchangeType {
    Binance, 
    Bybit,
    KuCoin,
    Unknown
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
) {

    let binance_book = LocalOrderBook::new();
    let binance_book_cl = binance_book.clone();

    // tokio::spawn(async move {
    //     binance_ws::connect(binance_book_cl).await;
    // });

    let kucoin_websocket = KuCoinWebsocket::new(true);

    let bybit_websocket = BybitWebsocket::new(false);
    let bybit_book = bybit_websocket.get_local_book();
    
    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let client = client.clone();

        // Proccesing long exchange
        tokio::spawn({
            let binance_book = binance_book.clone();
            let bybit_book = bybit_book.clone();
            let kucoin = kucoin_websocket.clone();
            let token = token.clone();
            let client = client.clone();

            async move {
                if long_exchange != ExchangeType::Unknown {
                    let ticker_tx = match long_exchange {
                        ExchangeType::Binance => return ,
                        ExchangeType::Bybit => return,
                        ExchangeType::KuCoin => kucoin.ticker_tx.clone(),
                        ExchangeType::Unknown => return,
                    };

                    ticker_tx.send((client.uuid.to_string().clone(), client.ticker.to_string())).await.unwrap();
                    let (snapshot_tx, mut snapshot_rx) = mpsc::unbounded_channel();
                    
                    tokio::spawn(async move {
                        kucoin.get_snapshot(snapshot_tx).await;
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

        // Proccesing short exchange
        tokio::spawn({
            let binance_book = binance_book.clone();
            let bybit_book = bybit_book.clone();
            // let kucoin_book = kucoin_book.clone();
            
            let token = token.clone();
            let client = client.clone();

            async move {
                if short_exchange != ExchangeType::Unknown {
                    // let exchange_book = match short_exchange {
                    //     ExchangeType::Binance => binance_book.clone(),
                    //     ExchangeType::Bybit => bybit_book.clone(),
                    //     ExchangeType::KuCoin => kucoin_book.clone(),
                    //     ExchangeType::Unknown => return 
                    // };

                    // tokio::spawn({
                    //     let token = token.clone();
                    //     let mut client = client.clone();

                    //     async move {
                    //         loop {
                    //             tokio::select! {
                    //                 _ = token.cancelled() => break,
                    //                 _ = client.send_snapshot(OrderType::Short, exchange_book.clone()) => {
                    //             }
                    //         }
                    //     }
                    // }});
                }
            }
        });
    }
}
