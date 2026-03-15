use std::sync::Arc;

use tokio::sync::{mpsc};

use crate::{adapters::{bybit_adapter::BybitAdapter, gate_adapter::GateAdapter, kucoin_adapter::KuCoinAdapter}, models::exchange::ExchangeType, services::{data_aggregator::DataAggregatorCmd, exchange_setup::ExchangeSetup}};

pub async fn run_ws_exchanges(
    data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>,
) {
    ExchangeSetup::new(
        ExchangeType::Bybit,
        Arc::new(BybitAdapter),
        true,
        data_aggregator_tx.clone()
    ).start();

    ExchangeSetup::new(
        ExchangeType::Gate,
        Arc::new(GateAdapter),
        true,
        data_aggregator_tx.clone()
    ).start();

    ExchangeSetup::new(
        ExchangeType::KuCoin,
        Arc::new(KuCoinAdapter),
        false,
        data_aggregator_tx.clone()
    ).start();
}
