use std::{sync::Arc, time::Duration};
use ordered_float::OrderedFloat;
use tokio::sync::{mpsc};
use crate::{exchanges::{bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket}, models::exchange::{ExchangeType, SharedSpreads, Spread}, services::market_manager::ExchangeWebsocket};

pub fn spawn_local_spread_engine(
    bybit: Arc<BybitWebsocket>,
    gate: Arc<GateWebsocket>,
    kucoin: Arc<KuCoinWebsocket>,
    shared_spreads: Arc<SharedSpreads>
) {
    let (spread_tx, mut spread_rx) = mpsc::channel::<Option<(ExchangeType, Option<f64>, Option<f64>)>>(1);

    tokio::spawn({
        async move {
            loop {
                bybit.clone().get_spread(spread_tx.clone()).await;
                gate.clone().get_spread(spread_tx.clone()).await;
                kucoin.clone().get_spread(spread_tx.clone()).await;
            }
        }
    });

    tokio::spawn({
        async move {
            while let Some(Some((exchange_type, ask, bid))) = spread_rx.recv().await {
                let spreads = shared_spreads.clone();

                let ask = match ask {
                    Some(a) => a,
                    None => continue
                };

                let bid = match bid {
                    Some(b) => b,
                    None => continue
                };
                
                spreads.exchange.insert(exchange_type, Spread {
                    ask: OrderedFloat(ask),
                    bid: OrderedFloat(bid),
                });
            }
        }
    });
}


pub fn calculate_spread_for_chart(
    shared_spreads: Arc<SharedSpreads>,
    spread_tx: Arc<tokio::sync::broadcast::Sender<(String, OrderedFloat<f64>)>>
) {
    tokio::spawn(async move {
        let spread_tx = spread_tx.clone();
        loop {
            let spreads = shared_spreads.clone();
            let exchanges: Vec<(ExchangeType, Spread)> = spreads.exchange.iter().map(|item| {
                (*item.key(), *item.value())
            }).collect();

            for i in 0..exchanges.len() {
                for j in (i+1)..exchanges.len() {
                    let (exchange_a, spread_a) = exchanges[i];
                    let (exchange_b, spread_b) = exchanges[j];

                    // let mid_price_1 = (spread_a.ask+spread_b.bid) / 2.0;
                    let spread_1 = (spread_a.ask - spread_b.bid) / spread_b.bid * 100.0;

                    if spread_tx.send((format!("{}/{}", exchange_a, exchange_b), spread_1)).is_err() {
                        continue;
                    }
                    // println!("{:?} -> {:?} : spread = {:.4}%", exchange_a, exchange_b, spread_1);
                    // println!("{:?} -> {:?} : {}; {}", exchange_a, exchange_b, spread_a.ask, spread_b.bid);

                    // let mid_price_2 = (spread_a.ask+spread_b.bid) / 2.0;
                    let spread_2 = (spread_b.ask - spread_a.bid) / spread_a.bid * 100.0;

                    if spread_tx.send((format!("{}/{}", exchange_b, exchange_a), spread_2)).is_err() {
                        continue;
                    }

                    // println!("{:?} -> {:?} : spread = {:.4}%", exchange_b, exchange_a, spread_2);
                    // println!("{:?} -> {:?} : {}; {}", exchange_b, exchange_a, spread_b.ask, spread_a.bid);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
}