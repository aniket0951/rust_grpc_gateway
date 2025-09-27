#![doc = include_str!("../README.md")]

use self::circuitbreaker::breaker::{CircuitBreaker, CircuitBreakerConfig};
use self::gateway::gateway::GrpcGateway;
use self::registry::service_registry::{RegistryTrait, ServiceRegistry};
use self::utils::errors::ResponseErrors;
use self::utils::model;
use self::utils::response::Response;
use self::utils::validation_errors::ValidationError;
use lazy_static::lazy_static;
use reqwest::StatusCode;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use std::time::Duration;

pub mod circuitbreaker;
pub mod discriptor;
pub mod gateway;
pub mod registry;
pub mod utils;

lazy_static! {
    static ref grpc_client_map: Mutex<HashMap<String, GrpcGateway>> = Mutex::new(HashMap::new());
}

pub struct Gateway {
    pub service_registry: ServiceRegistry,
    pub breaker: CircuitBreaker,
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
            breaker: CircuitBreaker::new(CircuitBreakerConfig {
                failure_threshold: 3,
                recovery_timeout: Duration::from_secs(3),
                half_open_max_calls: 2,
            }),
        }
    }
    pub async fn invoker(&self, req: model::RequestType) -> Response {
        let service = self.service_registry.discover(req.service.to_string());

        if service.is_none() {
            return Response {
                message: ResponseErrors::ServiceNotRegister(req.service.to_string()).message(),
                status: ResponseErrors::Error.message(),
                data: None,
                status_code: StatusCode::BAD_REQUEST,
            };
        }

        let service_config = service.unwrap();
        // should check the circute breaker is allowing or not to call the api
        let grpc_client = self.get_client(&service_config.endpoint).await;
        if grpc_client.is_err() {
            let e = grpc_client.err().unwrap();

            if e.to_string().to_lowercase().contains("transport error") {
                return Response {
                    message: ResponseErrors::TransportFailure.message(),
                    status: ResponseErrors::Error.message(),
                    data: None,
                    status_code: StatusCode::BAD_GATEWAY,
                };
            }
            return Response {
                message: std::borrow::Cow::Owned(e.to_string()),
                status: ResponseErrors::Error.message(),
                data: None,
                status_code: StatusCode::BAD_REQUEST,
            };
        }
        let client = grpc_client.unwrap();
        let breaker = service_config.breaker.clone().unwrap();
        let result = breaker
            .call(|| async move {
                let res = client
                    .invoke(
                        &req.service,
                        &req.method,
                        req.data.clone(),
                        service_config.clone(),
                    )
                    .await?;
                Ok(res)
            })
            .await;
        match result {
            Ok(response) => {
                let converted_data = serde_json::from_value(response).ok();
                Response {
                    message: ResponseErrors::Success.message(),
                    status: ResponseErrors::Success.message(),
                    data: converted_data,
                    status_code: StatusCode::OK,
                }
            }
            Err(e) => {
                if e.to_string().to_lowercase().contains("status: unavailable") {
                    return Response {
                        message: ResponseErrors::ServiceUnAvailable.message(),
                        status: ResponseErrors::Error.message(),
                        data: None,
                        status_code: StatusCode::SERVICE_UNAVAILABLE,
                    };
                }
                Response {
                    message: std::borrow::Cow::Owned(e.to_string()),
                    status: ResponseErrors::Error.message(),
                    data: None,
                    status_code: StatusCode::BAD_REQUEST,
                }
            }
        }
    }

    async fn get_client(&self, service_endpoint: &str) -> Result<GrpcGateway, Box<dyn Error>> {
        let mut grpc_client = match grpc_client_map.lock() {
            Ok(mp) => mp.get(service_endpoint).cloned(),
            Err(_) => None,
        };

        if grpc_client.is_none() {
            let result = GrpcGateway::new(service_endpoint).await;
            if result.is_err() {
                return Err(Box::new(ValidationError(result.err().unwrap().to_string())));
            }
            grpc_client = Some(result.unwrap());

            // will store the refernce of client connection
            if let Ok(mut mp) = grpc_client_map.lock() {
                mp.insert(service_endpoint.to_string(), grpc_client.clone().unwrap());
            }
        }
        Ok(grpc_client.unwrap())
    }
}
