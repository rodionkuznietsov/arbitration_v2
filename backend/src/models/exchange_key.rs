use serde::{Deserialize, Serialize};

use crate::models::websocket::Symbol;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiKeyResponse {
    #[serde(rename="data")]
    pub data: Data
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    #[serde(rename="token")]
    pub token: Symbol
}