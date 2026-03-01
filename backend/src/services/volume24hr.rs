use std::{sync::Arc, time::Duration};

use tokio::sync::broadcast;

use crate::{exchanges::{bybit_ws::BybitWebsocket, gate_rs::GateWebsocket}, models::exchange::ExchangeType};

pub struct ExchangeVolume {
    bybit: Arc<BybitWebsocket>,
    gate: Arc<GateWebsocket>,
    pub volume_tx: broadcast::Sender<(ExchangeType, String, f64)>
}

impl ExchangeVolume {
    pub fn new(
        bybit: Arc<BybitWebsocket>,
        gate: Arc<GateWebsocket>
    ) -> Self {
        let (volume_tx, _) = broadcast::channel::<(ExchangeType, String, f64)>(1000);

        Self { 
            bybit: bybit, 
            gate: gate, 
            volume_tx: volume_tx
        }
    }

    pub fn spawn_volume_engine(&self) {
        tokio::spawn({
            let volume_tx = self.volume_tx.clone();
            let bybit = self.bybit.clone();
            let gate = self.gate.clone();
            
            async move {
                loop {
                    // bybit.clone().get_volume24hr(volume_tx.clone()).await;
                    gate.clone().get_volume24hr(volume_tx.clone()).await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        });
    }
}