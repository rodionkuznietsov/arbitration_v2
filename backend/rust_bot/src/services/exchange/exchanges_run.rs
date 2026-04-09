use std::sync::Arc;

use tokio::sync::{mpsc};

use crate::{adapters::{bybit_adapter::BybitAdapter, gate_adapter::GateAdapter, kucoin_adapter::KuCoinAdapter}, models::exchange::ExchangeType, services::{data_aggregator::DataAggregatorCmd, exchange::{exchange_channel_store::ExchangeChannelStoreCmd, exchange_setup::ExchangeSetup}}};

pub async fn run_ws_exchanges(
    data_aggregator_tx: mpsc::Sender<DataAggregatorCmd>,
    exchange_channel_store_tx: mpsc::Sender<ExchangeChannelStoreCmd>
) {
    ExchangeSetup::new(
        ExchangeType::Bybit,
        Arc::new(BybitAdapter),
        true,
        data_aggregator_tx.clone(),
        exchange_channel_store_tx.clone()
    ).start();

    ExchangeSetup::new(
        ExchangeType::Gate,
        Arc::new(GateAdapter),
        false,
        data_aggregator_tx.clone(),
        exchange_channel_store_tx.clone()
    ).start();

    ExchangeSetup::new(
        ExchangeType::KuCoin,
        Arc::new(KuCoinAdapter),
        false,
        data_aggregator_tx.clone(),
        exchange_channel_store_tx.clone()
    ).start();
}
