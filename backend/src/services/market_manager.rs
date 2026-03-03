use tokio::sync::{broadcast, mpsc, oneshot};
use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::{aggregator::AggregatorEvent, orderbook::MarketType, websocket::{ChannelSubscription, ChannelType, ChartEvent, ServerToClientEvent}}, services::aggregator::{Aggregator, AggregatorCommand}, transport::ws::ConnectedClient};

pub async fn run_websockets(
    receiver: async_channel::Receiver<ConnectedClient>,
    pool: sqlx::PgPool
) {
    let (aggregator_tx, aggregator_rx) = mpsc::channel(100);
    let (lines_cache_tx, _) = broadcast::channel(1);

    let aggregator = Aggregator::new(
        aggregator_rx, 
        pool.clone(),
        lines_cache_tx.clone(),
    );
    tokio::spawn(aggregator.run());

    BybitWebsocket::new(true, aggregator_tx.clone());
    GateWebsocket::new(true, aggregator_tx.clone());
    KuCoinWebsocket::new(false, aggregator_tx.clone());
    BinXWebsocket::new(false, aggregator_tx.clone());
    MexcWebsocket::new(false, aggregator_tx.clone());
    BinanceWebsocket::new(false, aggregator_tx.clone());
    LBankWebsocket::new(false, aggregator_tx.clone());

    loop {
        if let Ok(client) = receiver.recv().await {
            let client = client.clone();
            let subscriptions = client.subscriptions.clone();
            let ticker = client.ticker.clone();
            let aggregator_tx = aggregator_tx.clone();
            
            for channel_subscriptions in subscriptions.values() {
                match channel_subscriptions {
                    ChannelSubscription::Chart { 
                        events,
                        long_exchange,
                        short_exchange
                    } => {
                        let mut client = client.clone();
                        let token = client.token.clone();

                        let ticker = ticker.clone();

                        if events.contains(&ChartEvent::Volume24hr) {
                            let (tx, rx) = oneshot::channel();

                            // Делаем подписку на volume
                            aggregator_tx.send(AggregatorCommand::Subscribe { 
                                event: AggregatorEvent::ChartEvent(ChartEvent::Volume24hr),
                                long_exchange: long_exchange.clone(), 
                                short_exchange: short_exchange.clone(), 
                                ticker: ticker.clone(),
                                response_tx: tx
                            }).await.ok();

                            if let Ok(mut sub_rx) = rx.await {
                                tokio::spawn(async move {
                                    loop {
                                        match sub_rx.recv().await {
                                            Ok(msg) => {
                                                
                                                let long_vol = msg.long_value;
                                                let short_vol = msg.short_value;

                                                client.send_to_client(
                                                    ServerToClientEvent::Volume24hr(
                                                        ChartEvent::Volume24hr,
                                                        ticker.clone(),
                                                        long_vol.unwrap(),
                                                        MarketType::Long
                                                    )
                                                ).await;

                                                client.send_to_client(
                                                    ServerToClientEvent::Volume24hr(
                                                        ChartEvent::Volume24hr,
                                                        ticker.clone(),
                                                        short_vol.unwrap(),
                                                        MarketType::Short
                                                    )
                                                ).await;
                                            },
                                            Err(broadcast::error::RecvError::Closed) => {
                                                println!("Клиент отключился");
                                                break;
                                            },
                                            _ => {}
                                        }
                                    }
                                });
                            }
                        }
                    },
                    ChannelSubscription::OrderBook { 
                        long_exchange, 
                        short_exchange 
                    } => {
                            let ticker = ticker.clone();
                            
                            let (tx, rx) = oneshot::channel();
                            aggregator_tx.send(AggregatorCommand::Subscribe { 
                                event: AggregatorEvent::OrderBook,
                                long_exchange: long_exchange.clone(), 
                                short_exchange: short_exchange.clone(), 
                                ticker: ticker.clone(),
                                response_tx: tx
                            }).await.ok();

                            let mut client = client.clone();
                            if let Ok(mut sub_rx) = rx.await {
                                tokio::spawn(async move {
                                    loop {
                                        match sub_rx.recv().await {
                                            Ok(msg) => {
                                                let long_book = msg.long_order_book.unwrap();
                                                let short_book = msg.short_order_book.unwrap();

                                                client.send_to_client(
                                                    ServerToClientEvent::OrderBook(
                                                        ChannelType::OrderBook, 
                                                        MarketType::Long,
                                                        long_book,
                                                        ticker.clone()
                                                    )
                                                ).await;

                                                client.send_to_client(
                                                    ServerToClientEvent::OrderBook(
                                                        ChannelType::OrderBook, 
                                                        MarketType::Short,
                                                        short_book,
                                                        ticker.clone()
                                                    )
                                                ).await;
                                            },
                                            Err(broadcast::error::RecvError::Closed) => {
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                });
                            }
                    }
                }
            }
        }
    }
}
