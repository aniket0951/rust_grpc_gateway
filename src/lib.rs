#![doc = include_str!("../README.md")]

use self::gateway::gateway::GrpcGateway;
use self::registery::model::{AuthType, RefreshTokenResponse, ServiceConfig};
use self::registery::service_registry::{RegistryTrait, ServiceRegistry};
use self::utils::errors::ResponseErrors;
use self::utils::model;
use self::utils::response::Response;
use self::utils::validation_errors::ValidationError;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde_json::json;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod discriptor;
pub mod gateway;
pub mod registery;
pub mod utils;

lazy_static! {
    static ref grpc_client_map: Mutex<HashMap<String, GrpcGateway>> = Mutex::new(HashMap::new());
}

pub struct Gateway {
    pub service_registry: ServiceRegistry,
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}

impl Gateway {
    pub fn new() -> Self {
        Self {
            service_registry: ServiceRegistry {},
        }
    }
    pub async fn refresh_invoker(
        &self,
        service_config: ServiceConfig,
    ) -> Result<(), Box<dyn Error>> {
        let mut grpc_client = match grpc_client_map.lock() {
            Ok(mp) => mp.get(&service_config.endpoint.to_string()).cloned(),
            Err(_) => None,
        };

        if grpc_client.is_none() {
            let client = match GrpcGateway::new(service_config.endpoint.as_str()).await {
                Ok(client) => client,
                Err(e) => {
                    if e.to_string().to_lowercase().contains("transport error") {
                        return Err(Box::new(ValidationError(
                            ResponseErrors::TransportFailure.to_string(),
                        )));
                    }
                    return Err(Box::new(ValidationError(e.to_string())));
                }
            };
            if let Ok(mut mp) = grpc_client_map.lock() {
                mp.insert(service_config.endpoint.to_string(), client.clone());
                grpc_client = Some(client)
            }
            // will store the refernce of client connection
        }
        let client = grpc_client.unwrap();
        match service_config.auth_config {
            Some(config) => match config.auth_refresh_config {
                Some(mut refresh_config) => {
                    match client
                        .refresh_oauth(
                            &refresh_config.service_name,
                            &refresh_config.method,
                            json!({
                                "refresh_token": refresh_config.refresh_token,
                            }),
                        )
                        .await
                    {
                        Ok(response) => {
                            // should update oauth config
                            let (access_token, refresh_token, expired_at) = (
                                response.get("access_token"),
                                response.get("refresh_token"),
                                response.get("expired_at"),
                            );

                            if let Some(token) = access_token {
                                refresh_config.access_token =
                                    token.as_str().unwrap_or_default().to_string();
                            }

                            if let Some(token) = refresh_token {
                                refresh_config.refresh_token =
                                    token.as_str().unwrap_or_default().to_string();
                            }

                            if let Some(expires) = expired_at {
                                refresh_config.expired_at = expires.as_u64().unwrap_or_default();
                            }
                        }
                        Err(e) => {
                            return Err(Box::new(ValidationError(e.to_string())));
                        }
                    }
                }
                None => {
                    return Err(Box::new(ValidationError(String::from(
                        "refresh config not availabel",
                    ))));
                }
            },
            None => {
                return Err(Box::new(ValidationError(String::from(
                    "refresh config not availabel",
                ))));
            }
        };
        Ok(())
    }
    pub async fn invoker(&self, req: model::RequestType) -> Response {
        let service = self.service_registry.discover(req.service.to_string());

        if service.is_none() {
            return Response {
                message: ResponseErrors::ServiceNotRegister(req.service.to_string()).to_string(),
                status: ResponseErrors::Error.to_string(),
                data: None,
                status_code: StatusCode::BAD_REQUEST,
            };
        }

        let service_config = service.unwrap();

        let mut grpc_client = match grpc_client_map.lock() {
            Ok(mp) => mp.get(&service_config.endpoint.to_string()).cloned(),
            Err(_) => None,
        };

        if grpc_client.is_none() {
            let client = match GrpcGateway::new(service_config.endpoint.as_str()).await {
                Ok(client) => client,
                Err(e) => {
                    if e.to_string().to_lowercase().contains("transport error") {
                        return Response {
                            message: ResponseErrors::TransportFailure.to_string(),
                            status: ResponseErrors::Error.to_string(),
                            data: None,
                            status_code: StatusCode::BAD_GATEWAY,
                        };
                    }
                    return Response {
                        message: e.to_string(),
                        status: ResponseErrors::Error.to_string(),
                        data: None,
                        status_code: StatusCode::BAD_REQUEST,
                    };
                }
            };
            // will store the refernce of client connection
            if let Ok(mut mp) = grpc_client_map.lock() {
                mp.insert(service_config.endpoint.to_string(), client.clone());
                grpc_client = Some(client)
            }
        }

        let client = grpc_client.unwrap();

        // check for auth is valid or not
        if service_config.auth_config.is_some() {
            let oauth_config = service_config.clone().auth_config.unwrap();
            match oauth_config.auth_type {
                AuthType::APIKey {
                    header_name: _,
                    value: _,
                } => {}
                AuthType::JWTToken {
                    header_name: _,
                    value: _,
                } => {
                    if oauth_config.auth_refresh_config.is_some() {
                        let refresh_config = oauth_config.auth_refresh_config.unwrap().clone();
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        if now > refresh_config.expired_at {
                            // need to refresh the token
                            let refresh_resp = self.refresh_invoker(service_config.clone()).await;

                            if refresh_resp.is_err() {
                                return Response {
                                    message: ResponseErrors::InternalServerError.to_string(),
                                    status: ResponseErrors::Error.to_string(),
                                    data: None,
                                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                                };
                            }
                        }
                    }
                }
            }
        }

        match client
            .invoke(&req.service, &req.method, req.data, service_config)
            .await
        {
            Ok(response) => {
                let converted_data = serde_json::from_value(response).ok();
                Response {
                    message: ResponseErrors::Success.to_string(),
                    status: ResponseErrors::Success.to_string(),
                    data: converted_data,
                    status_code: StatusCode::OK,
                }
            }
            Err(e) => {
                if e.to_string().to_lowercase().contains("status: unavailable") {
                    return Response {
                        message: ResponseErrors::ServiceUnAvailable.to_string(),
                        status: ResponseErrors::Error.to_string(),
                        data: None,
                        status_code: StatusCode::SERVICE_UNAVAILABLE,
                    };
                }
                Response {
                    message: e.to_string(),
                    status: ResponseErrors::Error.to_string(),
                    data: None,
                    status_code: StatusCode::BAD_REQUEST,
                }
            }
        }
    }
}
