use std::time;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AuthType {
    APIKey { header_name: String, value: String },
    JWTToken { header_name: String, value: String },
}

#[derive(Debug, Clone)]
pub struct AuthRefreshConfig {
    pub service_name: String,
    pub method: String,
    pub access_token: String,
    pub expired_at: u64,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct InternalAuthConfig {
    pub auth_type: AuthType,
    pub auth_refresh_config: Option<AuthRefreshConfig>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub endpoint: String,
    pub service_name: String,
    pub auth_config: Option<InternalAuthConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expired_at: u64,
}

impl ServiceConfig {
    pub fn update_auth_refresh_config(&self, auth_refresh_config: AuthRefreshConfig) {}
}
