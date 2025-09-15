use async_trait::async_trait;
use std::error::Error;

use crate::{
    registry::{api_key::APIKeyAuth, jwt_token::JWTTokenAuth},
    utils::validation_errors::ValidationError,
};

pub trait Auth: Send + Sync + std::fmt::Debug {
    fn header_name(&self) -> &str;
    fn value(&self) -> String;
    fn requires_refresh(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    APIKeyAuth(APIKeyAuth),
    JWTTokenAuth(JWTTokenAuth),
}

impl Auth for AuthConfig {
    fn header_name(&self) -> &str {
        match self {
            AuthConfig::APIKeyAuth(apikey_auth) => apikey_auth.header_name.as_str(),
            AuthConfig::JWTTokenAuth(jwttoken_auth) => jwttoken_auth.header_name.as_str(),
        }
    }

    fn value(&self) -> String {
        match self {
            AuthConfig::APIKeyAuth(apikey_auth) => apikey_auth.value().to_string(),
            AuthConfig::JWTTokenAuth(jwttoken_auth) => jwttoken_auth.access_token.to_string(),
        }
    }
}

#[async_trait]
pub trait Refreshable {
    async fn refresh_if_expired(
        &mut self,
        service_endpoint: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl Refreshable for AuthConfig {
    async fn refresh_if_expired(
        &mut self,
        service_endpoint: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        match self {
            AuthConfig::APIKeyAuth(_) => Ok(self.value()), // APIKey doesn't refresh
            AuthConfig::JWTTokenAuth(jwt) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();

                if now < jwt.expired_at {
                    let result = jwt.refresh_token(service_endpoint).await;
                    if result.is_err() {
                        return Err(Box::new(ValidationError(result.err().unwrap().to_string())));
                    }
                    let access_token = result.unwrap();
                    Ok(access_token)
                } else {
                    Ok(jwt.access_token.clone())
                }
            }
        }
    }
}
