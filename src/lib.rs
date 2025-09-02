#![doc = include_str!("../README.md")]

use self::gateway::gateway::GrpcGateway;
use self::registery::service_registry::{RegistryTrait, ServiceRegistry};
use self::utils::errors::ResponseErrors;
use self::utils::model;
use self::utils::response::Response;

use lazy_static::lazy_static;
use reqwest::StatusCode;

use std::collections::HashMap;
use std::sync::Mutex;

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

impl Gateway {
    pub fn new() -> Self {
        Self {
            service_registry: ServiceRegistry {},
        }
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
            let mp_result = grpc_client_map.lock();
            if mp_result.is_ok() {
                mp_result
                    .unwrap()
                    .insert(service_config.endpoint.to_string(), client.clone());
            };

            grpc_client = Some(client)
        }

        let client = grpc_client.unwrap();
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
