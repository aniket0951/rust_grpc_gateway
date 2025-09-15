use serde::{Deserialize, Serialize};

use crate::registry::auth::AuthConfig;

#[derive(Debug, Clone)]
pub enum AuthType {
    APIKey,
    JWTToken,
}

#[derive(Debug, Clone)]
pub struct AuthRefreshConfig {
    pub service_name: String,
    pub method: String,
    pub header_name: String,
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
    //pub auth_config: Option<Arc<dyn Auth>>,
    pub auth_config: Option<AuthConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshAuthTokenJson {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "refreshToken")]
    pub refresh_token: String,

    #[serde(rename = "expiredAt", deserialize_with = "string_to_u64")]
    pub expired_at: u64,
}

// custom deserializer for expiredAt since it’s a string in response
fn string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}
