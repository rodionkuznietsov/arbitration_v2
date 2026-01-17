use crate::{websocket::ConnectedClient, exchanges::{orderbook::{LocalOrderBook, OrderType}, *}};

#[derive(Debug, Clone, PartialEq)]
pub enum ExchangeType {
    Binance, 
    Bybit,
    Unknown
}

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
) {

    let binance_book = LocalOrderBook::new();
    // let binance_book_cl = binance_book.clone();

    let bybit_book = LocalOrderBook::new();
    let bybit_book_cl = bybit_book.clone();

    // tokio::spawn(async move {
    //     binance_ws::connect(binance_book_cl).await;
    // });

    tokio::spawn(async move {
        bybit_ws::connect("spot", bybit_book_cl).await;
    });

    
    while let Ok(client) = receiver.recv().await {  
        let token = client.token.clone();
        let long_exchange = client.long_exchange.clone();
        let short_exchange = client.short_exchange.clone();
        let client = client.clone();

        // Proccesing long exchange
        tokio::spawn({
            let binance_book = binance_book.clone();
            let bybit_book = bybit_book.clone();
            let token = token.clone();
            let client = client.clone();

            async move {
                if long_exchange != ExchangeType::Unknown {
                    let exchange_book = match long_exchange {
                        ExchangeType::Binance => binance_book.clone(),
                        ExchangeType::Bybit => bybit_book.clone(),
                        ExchangeType::Unknown => return,
                    };

                    tokio::spawn({
                        let token = token.clone();
                        let mut client = client.clone();

                        async move {
                            loop {
                                tokio::select! {
                                    _ = token.cancelled() => break,
                                    _ = client.send_snapshot(OrderType::Long, exchange_book.clone()) => {
                                }
                            }
                        }
                    }});
                }
            }
        });

        // Proccesing short exchange
        tokio::spawn({
            let binance_book = binance_book.clone();
            let bybit_book = bybit_book.clone();
            let token = token.clone();
            let client = client.clone();

            async move {
                if short_exchange != ExchangeType::Unknown {
                    let exchange_book = match short_exchange {
                        ExchangeType::Binance => binance_book.clone(),
                        ExchangeType::Bybit => bybit_book.clone(),
                        ExchangeType::Unknown => return 
                    };

                    tokio::spawn({
                        let token = token.clone();
                        let mut client = client.clone();

                        async move {
                            loop {
                                tokio::select! {
                                    _ = token.cancelled() => break,
                                    _ = client.send_snapshot(OrderType::Short, exchange_book.clone()) => {
                                }
                            }
                        }
                    }});
                }
            }
        });
    }
}
