use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestType {
    pub method: String,
    pub service: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceRegisterRequest {
    pub service_name: String,
    pub host: String,
    pub port: String,
    pub health_check_endpoint: String,
}
