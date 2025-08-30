use prost_types::Struct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Response {
    pub message: String,
    pub status: String,
    pub data: Option<serde_json::Value>,
}
