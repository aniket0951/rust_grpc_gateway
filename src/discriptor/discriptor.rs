use prost::Message;
use prost_types::FileDescriptorProto;
use std::{collections::HashMap, time::Instant};

use anyhow::Result;
use prost_reflect::{DescriptorPool, MethodDescriptor, ServiceDescriptor};
use tokio::time;
use tonic::transport::Channel;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;

#[derive(Debug)]
pub struct CachedDescriptors {
    pub pool: DescriptorPool,
    pub services: HashMap<String, ServiceDescriptor>,
    pub methods: HashMap<String, MethodDescriptor>,
    pub last_updated_at: std::time::Instant,
}

impl CachedDescriptors {
    pub fn new() -> Self {
        Self {
            pool: DescriptorPool::new(),
            services: HashMap::new(),
            methods: HashMap::new(),
            last_updated_at: Instant::now(),
        }
    }

    pub async fn load_discriptor(&mut self, channel: Channel) -> Result<()> {
        println!("loading discriptor..");
        let mut reflection_client = ServerReflectionClient::new(channel);

        // get list of services first
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
                if let tonic_reflection::pb::v1::server_reflection_response::MessageResponse::ListServicesResponse(services_resp) = message_response {
                    for service in services_resp.service {
                        service_names.push(service.name);
                    }
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
                    if let tonic_reflection::pb::v1::server_reflection_response::MessageResponse::FileDescriptorResponse(fd_resp) = message_response {
                        for fd_bytes in fd_resp.file_descriptor_proto {
                            let file_descriptor = FileDescriptorProto::decode(fd_bytes.as_slice())?;
                            all_file_descriptors.push(file_descriptor);
                        }
                    }
                }
            }
        }

        let mut new_pool = DescriptorPool::new();
        for fd in all_file_descriptors {
            if let Err(e) = new_pool.add_file_descriptor_proto(fd) {
                return Err(anyhow::anyhow!("Failed to add file descriptor: {}", e));
            }
        }

        // Cache services and methods for fast lookup
        let mut new_services = HashMap::new();
        let mut new_methods = HashMap::new();

        for service in new_pool.services() {
            // Use fully-qualified service name so lookups with package work
            let service_full_name = service.full_name().to_string();
            new_services.insert(service_full_name.clone(), service.clone());

            // Cache all methods for this service with fully-qualified service key
            for method in service.methods() {
                let method_key = format!("{}.{}", service_full_name, method.name());
                new_methods.insert(method_key, method);
            }
        }

        // Update cache atomically
        self.pool = new_pool;
        self.services = new_services;
        self.methods = new_methods;
        self.last_updated_at = std::time::Instant::now();

        Ok(())
    }

    pub fn get_method(&self, service: &str, method: &str) -> Option<&MethodDescriptor> {
        let key = format!("{}.{}", service, method);
        self.methods.get(&key)
    }

    pub fn get_service(&self, service: &str) -> Option<&ServiceDescriptor> {
        self.services.get(service)
    }

    pub fn get_all_service(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    pub fn is_stable(&self, max_age: time::Duration) -> bool {
        self.last_updated_at.elapsed() > max_age
    }
}

impl Default for CachedDescriptors {
    fn default() -> Self {
        Self::new()
    }
}
