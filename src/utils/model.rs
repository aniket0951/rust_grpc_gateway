use serde::{Deserialize, Serialize};

use crate::registery::model::InternalAuthConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestType {
    pub method: String,
    pub service: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ServiceRegisterRequest {
    pub service_name: String,
    pub host: String,
    pub port: String,
    pub health_check_endpoint: String,
    pub oauth_config: InternalAuthConfig,
}
