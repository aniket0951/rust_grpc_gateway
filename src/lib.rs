use self::gateway::gateway::GrpcGateway;
use self::registery::service_registry::{RegistryTrait, ServiceRegistry};
use self::utils::errors::ResponseErrors;
use self::utils::model;
use self::utils::response::Response;

use lazy_static::lazy_static;

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
                status: String::from("faild"),
                data: None,
            };
        }

        let service_name = service.unwrap();

        let mut grpc_client = match grpc_client_map.lock() {
            Ok(mp) => mp.get(&service_name.to_string()).cloned(),
            Err(_) => None,
        };

        if grpc_client.is_none() {
            let result = GrpcGateway::new(service_name.as_str()).await;
            if result.is_err() {
                return Response {
                    message: result.err().unwrap().to_string(),
                    status: String::from("failed"),
                    data: None,
                };
            }
            let client = result.unwrap();
            // will store the refernce of client connection
            let mp_result = grpc_client_map.lock();
            if mp_result.is_ok() {
                mp_result
                    .unwrap()
                    .insert(service_name.to_string(), client.clone());
            };

            grpc_client = Some(client)
        }

        let client = grpc_client.unwrap();
        match client.invoke(&&req.service, &req.method, req.data).await {
            Ok(response) => {
                let converted_data = serde_json::from_value(response).ok();
                Response {
                    message: ResponseErrors::Success.to_string(),
                    status: String::from("success"),
                    data: converted_data,
                }
            }
            Err(e) => {
                if e.to_string().contains("status: Unavailable") {
                    return Response {
                        message: ResponseErrors::ServiceUnAvailable.to_string(),
                        status: String::from("faild"),
                        data: None,
                    };
                }
                Response {
                    message: e.to_string(),
                    status: String::from("failed"),
                    data: None,
                }
            }
        }
    }
}
