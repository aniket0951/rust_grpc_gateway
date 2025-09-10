use super::model::{AuthType, ServiceConfig};
use crate::gateway::gateway::GrpcGateway;
use crate::registery::model::{AuthRefreshConfig, InternalAuthConfig};
use crate::utils::errors::ResponseErrors;
use crate::utils::model::ServiceRegisterRequest;
use crate::utils::validation_errors::ValidationError;
use anyhow::Result;
use serde_json::json;
use std::error::Error;
use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_MAP: Mutex<HashMap<String, ServiceConfig>> = Mutex::new(HashMap::new());
}

pub trait RegistryTrait {
    fn validate_oauth_config(
        &self,
        oauth_config: InternalAuthConfig,
        service_endpoint: String,
    ) -> impl std::future::Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn register(
        &self,
        req: ServiceRegisterRequest,
    ) -> impl std::future::Future<Output = Result<Option<String>, Box<dyn Error>>> + Send;
    fn discover(&self, service_name: String) -> Option<ServiceConfig>;
    fn update_auth_config(
        &self,
        service_name: String,
        oauth_config: InternalAuthConfig,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct ServiceRegistry {}

impl RegistryTrait for ServiceRegistry {
    async fn register(
        &self,
        req: ServiceRegisterRequest,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let val = format!("http://{}:{}", req.host, req.port);
        let mut config = ServiceConfig {
            endpoint: val.to_string(),
            service_name: req.service_name.to_string(),
            auth_config: None,
        };

        let validation_res = self
            .validate_oauth_config(req.oauth_config.clone(), val.to_string())
            .await;

        if validation_res.is_err() {
            return Err(Box::new(ValidationError(
                validation_res.err().unwrap().to_string(),
            )));
        }

        config.auth_config = Some(req.oauth_config);

        match GLOBAL_MAP.lock() {
            Ok(mut mp) => {
                mp.insert(req.service_name.to_string(), config);
                Ok(Some(val))
            }
            Err(e) => Err(Box::new(ValidationError(e.to_string()))),
        }
    }

    fn discover(&self, service_name: String) -> Option<ServiceConfig> {
        match GLOBAL_MAP.lock() {
            Ok(mp) => mp.get(&service_name).cloned(),
            Err(_) => None,
        }
    }

    fn update_auth_config(
        &self,
        service_name: String,
        auth_config: InternalAuthConfig,
    ) -> Result<(), Box<dyn Error>> {
        match GLOBAL_MAP.lock() {
            Ok(mut mp) => {
                mp.entry(service_name.to_string())
                    .and_modify(|service_config| service_config.auth_config = Some(auth_config));
                Ok(())
            }
            Err(_) => todo!(),
        }
    }

    async fn validate_oauth_config(
        &self,
        oauth_config: InternalAuthConfig,
        service_endpoint: String,
    ) -> Result<(), Box<dyn Error>> {
        match oauth_config.auth_type {
            AuthType::APIKey {
                header_name: _,
                value: _,
            } => Ok(()),
            AuthType::JWTToken {
                header_name: _,
                value: _,
            } => {
                if oauth_config.auth_refresh_config.is_none() {
                    return Err(Box::new(ValidationError(
                        ResponseErrors::OAuthRefreshConfigMissingError.to_string(),
                    )));
                };

                let refresh_config = oauth_config.auth_refresh_config.unwrap();
                if refresh_config.service_name.is_empty() || refresh_config.method.is_empty() {
                    return Err(Box::new(ValidationError(
                        ResponseErrors::OAuthRefreshConfigMissingError.to_string(),
                    )));
                };

                // check the refrsh by invoking the endpoints
                match GrpcGateway::new(&service_endpoint).await {
                    Ok(current_gateway) => {
                        match current_gateway
                            .refresh_oauth(
                                &refresh_config.service_name,
                                &refresh_config.method,
                                json!({
                                    "refresh_token":refresh_config.refresh_token
                                }),
                            )
                            .await
                        {
                            Ok(_) => Ok(()),
                            Err(_) => Err(Box::new(ValidationError(String::from(
                                "faild to refresh oauth config",
                            )))),
                        }
                    }
                    Err(_) => Err(Box::new(ValidationError(
                        ResponseErrors::ServiceUnAvailable.to_string(),
                    ))),
                }
            }
        }
    }
}
