use crate::models::candle::Candle;
use crate::exchanges::orderbook::SnapshotUi;

pub struct UserState {
    pub candle_history: Vec<Candle>,
    pub snapshot: Option<SnapshotUi>,
}