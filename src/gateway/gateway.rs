use anyhow::Result;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DynamicMessage, ReflectMessage};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use tonic::metadata::{MetadataKey, MetadataValue};

use crate::discriptor::discriptor_manager::ReflectionDiscriptorManager;
use crate::gateway::dynamic_grpc_client::BytesCodec;
use crate::registry::auth::{Auth, Refreshable};
use crate::registry::model::ServiceConfig;
use crate::utils::validation_errors::ValidationError;

#[derive(Debug, Clone)]
pub struct GrpcGateway {
    discriptor_manager: Arc<ReflectionDiscriptorManager>,
}

impl GrpcGateway {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let manager = ReflectionDiscriptorManager::new(endpoint).await?;
        Ok(Self {
            discriptor_manager: Arc::new(manager),
        })
    }

    pub async fn invoke(
        &self,
        service: &str,
        method: &str,
        data: Value,
        service_config: ServiceConfig,
    ) -> Result<serde_json::Value> {
        // get method discriptor from cache
        let method_desc = self
            .discriptor_manager
            .get_method(service, method)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Method {}.{} not found", service, method))?;

        // create request using cached
        let mut request_message = DynamicMessage::new(method_desc.input());
        self.json_to_dynamic_message(&data, &mut request_message)?;

        // Encode request
        let request_bytes = request_message.encode_to_vec();
        let full_method_name = format!("/{}/{}", service, method);
        let mut request = tonic::Request::new(request_bytes);

        if let Some(mut config) = service_config.auth_config.clone() {
            // call refresh trait first
            let token = config.refresh_if_expired(&service_config.endpoint).await;

            if let Err(e) = token {
                return Err(anyhow::anyhow!(ValidationError(e.to_string())));
            }
            // insert into metadata
            let key = MetadataKey::from_bytes(config.header_name().as_bytes()).unwrap();
            let value = MetadataValue::from_str(config.value().as_str()).unwrap();
            request.metadata_mut().insert(key, value);
        }

        // create dynamic client using shared channel
        let channel = self.discriptor_manager.channel.clone();

        let mut client = tonic::client::Grpc::new(channel.clone());

        client
            .ready()
            .await
            .map_err(|e| anyhow::anyhow!("gRPC service not ready: {:?}", e))?;

        let response: tonic::Response<Bytes> = client
            .unary(request, full_method_name.parse()?, BytesCodec)
            .await?;

        let output_type = method_desc.output();
        let response_message = DynamicMessage::decode(output_type, response.into_inner())?;

        // Convert back to JSON
        let response_json = self.dynamic_message_to_json(&response_message)?;
        Ok(response_json)
    }

    fn json_to_dynamic_message(&self, json: &Value, message: &mut DynamicMessage) -> Result<()> {
        // Use prost-reflect's serde-powered deserializer to fully support nested
        // messages, arrays, enums, maps, oneofs, bytes, and canonical field names.
        let descriptor = message.descriptor();
        let json_string = serde_json::to_string(json)?;
        let mut deserializer = serde_json::Deserializer::from_str(&json_string);
        let parsed = DynamicMessage::deserialize(descriptor, &mut deserializer)?;
        deserializer.end()?;
        *message = parsed;
        Ok(())
    }

    fn dynamic_message_to_json(&self, message: &DynamicMessage) -> Result<Value> {
        // Use prost-reflect's serde support to convert to canonical Protobuf JSON
        let value = serde_json::to_value(message)?;
        Ok(value)
    }

    pub async fn refresh_oauth(
        &self,
        service: &str,
        method: &str,
        data: Value,
    ) -> Result<serde_json::Value> {
        let method_desc = self
            .discriptor_manager
            .get_method(service, method)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Method {}.{} not found", service, method))?;

        // create request using cached
        let mut request_message = DynamicMessage::new(method_desc.input());
        self.json_to_dynamic_message(&data, &mut request_message)?;

        // Encode request
        let request_bytes = request_message.encode_to_vec();
        let full_method_name = format!("/{}/{}", service, method);
        let request = tonic::Request::new(request_bytes);

        // create dynamic client using shared channel
        let channel = self.discriptor_manager.channel.clone();

        let mut client = tonic::client::Grpc::new(channel.clone());

        client
            .ready()
            .await
            .map_err(|e| anyhow::anyhow!("gRPC service not ready: {:?}", e))?;

        let response: tonic::Response<Bytes> = client
            .unary(request, full_method_name.parse()?, BytesCodec)
            .await?;

        let output_type = method_desc.output();
        let response_message = DynamicMessage::decode(output_type, response.into_inner())?;

        // Convert back to JSON
        let response_json = self.dynamic_message_to_json(&response_message)?;
        Ok(response_json)
    }
}
