use std::{sync::Arc, time::Duration};
use ordered_float::OrderedFloat;
use tokio::sync::{RwLock, mpsc};
use crate::{exchanges::{bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket}, models::exchange::{self, ExchangeType, SharedSpreads, Spread}, services::market_manager::ExchangeWebsocket};

pub fn spawn_local_spread_engine(
    bybit: Arc<BybitWebsocket>,
    gate: Arc<GateWebsocket>,
    kucoin: Arc<KuCoinWebsocket>,
    shared_spreads: Arc<RwLock<SharedSpreads>>
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
                // println!("{:?}: {:?}; {:?}", exchange_type, ask, bid);
                let mut write = shared_spreads.write().await;
                write.exchange.insert(exchange_type, Spread {
                    ask: OrderedFloat(ask.unwrap()),
                    bid: OrderedFloat(bid.unwrap()),
                });
            }
        }
    });
}


pub fn calculate_spread_for_chart(
    shared_spreads: Arc<RwLock<SharedSpreads>>,
) {
    tokio::spawn(async move {
        loop {
            let read = shared_spreads.read().await;

            let exchanges: Vec<(&ExchangeType, &Spread)> = read.exchange.iter().collect();

            for i in 0..exchanges.len() {
                for j in (i+1)..exchanges.len() {
                    let (exchange_a, spread_a) = exchanges[i];
                    let (exchange_b, spread_b) = exchanges[j];

                    let mid_price_1 = (spread_a.ask+spread_b.bid) / 2.0;
                    let spread_1 = (spread_a.ask - spread_b.bid) / mid_price_1 * 100.0;
                    println!("{:?} -> {:?} : spread = {:.4}%", exchange_a, exchange_b, spread_1);
                    // println!("{:?} -> {:?} : {}; {}", exchange_a, exchange_b, spread_a.ask, spread_b.bid);

                    let mid_price_2 = (spread_a.ask+spread_b.bid) / 2.0;
                    let spread_2 = (spread_b.ask - spread_a.bid) / mid_price_2 * 100.0;
                    println!("{:?} -> {:?} : spread = {:.4}%", exchange_b, exchange_a, spread_2);
                    // println!("{:?} -> {:?} : {}; {}", exchange_b, exchange_a, spread_b.ask, spread_a.bid);
                }
            }
        }
    });
}