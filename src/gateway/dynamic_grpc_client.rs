use anyhow::Result;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage};
use prost_types::FileDescriptorProto;
use serde_json::Value;

use bytes::{Buf, BufMut, Bytes};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::transport::Channel;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;

// Create a simple bytes codec
#[derive(Debug, Clone)]
pub struct BytesCodec;

impl Codec for BytesCodec {
    type Encode = Vec<u8>;
    type Decode = Bytes;

    type Encoder = BytesEncoder;
    type Decoder = BytesDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        BytesEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        BytesDecoder
    }
}

#[derive(Debug, Clone)]
pub struct BytesEncoder;

impl Encoder for BytesEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put_slice(&item);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BytesDecoder;

impl Decoder for BytesDecoder {
    type Item = Bytes;
    type Error = tonic::Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.remaining();
        if len == 0 {
            return Ok(None);
        }
        let mut buf = vec![0u8; len];
        src.copy_to_slice(&mut buf);
        Ok(Some(Bytes::from(buf)))
    }
}

#[derive(Clone)]
pub struct DynamicGrpcClient {
    channel: Channel,
    descriptor_pool: Option<DescriptorPool>,
}

impl DynamicGrpcClient {
    pub async fn new(service_url: String) -> Result<Self> {
        let channel = Channel::from_shared(service_url.clone())?.connect().await?;

        Ok(Self {
            channel,
            descriptor_pool: None,
        })
    }

    // Load service descriptors using reflection
    pub async fn load_descriptors(&mut self) -> Result<()> {
        let mut reflection_client = ServerReflectionClient::new(self.channel.clone());

        // Get list of services first
        let list_services_request = tonic_reflection::pb::v1::ServerReflectionRequest {
            message_request: Some(
                tonic_reflection::pb::v1::server_reflection_request::MessageRequest::ListServices(
                    String::new(),
                ),
            ),
            host: String::from(""),
        };

        let response = reflection_client
            .server_reflection_info(futures::stream::iter(vec![list_services_request]))
            .await?;

        let mut response_stream = response.into_inner();
        let mut service_names = Vec::new();

        // Extract service names
        while let Some(resp) = response_stream.message().await? {
            if let Some(message_response) = resp.message_response {
                match message_response {
                    tonic_reflection::pb::v1::server_reflection_response::MessageResponse::ListServicesResponse(services_resp) => {
                        for service in services_resp.service {
                            service_names.push(service.name);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Now get file descriptors for each service
        let mut all_file_descriptors = Vec::new();

        for service_name in service_names {
            let file_containing_symbol_request = tonic_reflection::pb::v1::ServerReflectionRequest {
                message_request: Some(
                    tonic_reflection::pb::v1::server_reflection_request::MessageRequest::FileContainingSymbol(service_name)
                ),
                host:String::from(""),
            };

            let response = reflection_client
                .server_reflection_info(futures::stream::iter(vec![file_containing_symbol_request]))
                .await?;

            let mut response_stream = response.into_inner();

            while let Some(resp) = response_stream.message().await? {
                if let Some(message_response) = resp.message_response {
                    match message_response {
                        tonic_reflection::pb::v1::server_reflection_response::MessageResponse::FileDescriptorResponse(fd_resp) => {
                            for fd_bytes in fd_resp.file_descriptor_proto {
                                let file_descriptor = FileDescriptorProto::decode(fd_bytes.as_slice())?;
                                all_file_descriptors.push(file_descriptor);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Build descriptor pool from FileDescriptorProto
        let mut pool = DescriptorPool::new();
        for fd in all_file_descriptors {
            if let Err(e) = pool.add_file_descriptor_proto(fd) {
                return Err(anyhow::anyhow!("Failed to add file descriptor: {}", e));
            }
        }
        self.descriptor_pool = Some(pool);

        Ok(())
    }

    pub async fn invoke_method(
        &mut self,
        service_name: &str,
        method_name: &str,
        request_json: Value,
    ) -> Result<Value> {
        // Ensure descriptors are loaded
        if self.descriptor_pool.is_none() {
            self.load_descriptors().await?;
        }

        let pool = self.descriptor_pool.as_ref().unwrap();

        // Find service and method
        let service = pool
            .get_service_by_name(service_name)
            .ok_or_else(|| anyhow::anyhow!("Service {} not found", service_name))?;

        let method = service
            .methods()
            .find(|m| m.name() == method_name)
            .ok_or_else(|| anyhow::anyhow!("Method {} not found", method_name))?;

        // Convert JSON to protobuf message
        let input_type = method.input();
        let mut request_message = DynamicMessage::new(input_type);

        // Parse JSON into dynamic message
        self.json_to_dynamic_message(&request_json, &mut request_message)?;

        // Encode message
        let request_bytes = request_message.encode_to_vec();

        // Make gRPC call
        let full_method_name = format!("/{}/{}", service_name, method_name);
        let mut request = tonic::Request::new(request_bytes);
        // Add auth header required by server interceptor
        request
            .metadata_mut()
            .insert("authorization", "Bearer some-secret-token".parse().unwrap());

        let mut client = tonic::client::Grpc::new(self.channel.clone());
        // Ensure the underlying service is ready before sending the request
        client
            .ready()
            .await
            .map_err(|e| anyhow::anyhow!("gRPC service not ready: {:?}", e))?;

        let response: tonic::Response<Bytes> = client
            .unary(request, full_method_name.parse()?, BytesCodec)
            .await?;

        // Decode response
        let output_type = method.output();
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
}
