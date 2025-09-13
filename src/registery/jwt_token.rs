use super::auth::Auth;
use crate::gateway::gateway::GrpcGateway;
use crate::grpc_client_map;

use crate::registery::model::RefreshAuthTokenJson;
use crate::utils::errors::ResponseErrors;
use crate::utils::validation_errors::ValidationError;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct JWTTokenAuth {
    pub header_name: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expired_at: u64,
    /// to refresh the expired token --> GRPC Call
    pub service_name: String,
    pub method: String,
}

impl Auth for JWTTokenAuth {
    fn header_name(&self) -> &str {
        &self.header_name
    }

    fn value(&self) -> String {
        self.access_token.to_string()
    }
}

impl JWTTokenAuth {
    pub async fn refresh_token(
        &mut self,
        service_endpoint: &str,
    ) -> Result<String, Box<dyn Error>> {
        let mut grpc_client = match grpc_client_map.lock() {
            Ok(mp) => mp.get(&service_endpoint.to_string()).cloned(),
            Err(_) => None,
        };

        if grpc_client.is_none() {
            let client = match GrpcGateway::new(service_endpoint).await {
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
                mp.insert(service_endpoint.to_string(), client.clone());
                grpc_client = Some(client)
            }
        }

        let client = grpc_client.unwrap();

        let response = client
            .refresh_oauth(
                &self.service_name,
                &self.method,
                serde_json::json_internal!({
                    "refresh_token":self.refresh_token,
                }),
            )
            .await;

        if response.is_err() {
            return Err(Box::new(ValidationError(
                response.err().unwrap().to_string(),
            )));
        }
        let data: RefreshAuthTokenJson = serde_json::from_value(response.unwrap())?;
        self.access_token = data.access_token.to_string();
        self.refresh_token = data.refresh_token.to_string();
        self.expired_at = data.expired_at;

        Ok(String::from(""))
    }
}
