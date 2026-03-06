use tokio::sync::{mpsc};

use crate::{exchanges::{binance_ws::BinanceWebsocket, binx_ws::BinXWebsocket, bybit_ws::BybitWebsocket, gate_rs::GateWebsocket, kucoin_ws::KuCoinWebsocket, lbank_ws::LBankWebsocket, mexc_ws::MexcWebsocket}, models::aggregator::ClientAggregatorCmd, services::aggregator::Aggregator};

pub async fn run_websockets(
    client_tx: mpsc::Sender<ClientAggregatorCmd>,
    pool: sqlx::PgPool
) {
    let (aggregator_tx, aggregator_rx) = mpsc::channel(1024);
    let aggregator = Aggregator::new(
        aggregator_rx, 
        pool.clone(),
    );
    tokio::spawn(aggregator.run(client_tx));

    BybitWebsocket::new(true, aggregator_tx.clone());
    GateWebsocket::new(true, aggregator_tx.clone());
    KuCoinWebsocket::new(false, aggregator_tx.clone());
    BinXWebsocket::new(false, aggregator_tx.clone());
    MexcWebsocket::new(false, aggregator_tx.clone());
    BinanceWebsocket::new(false, aggregator_tx.clone());
    LBankWebsocket::new(false, aggregator_tx.clone());
}
