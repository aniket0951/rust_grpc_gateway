use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Result;
use prost_reflect::{MethodDescriptor, ServiceDescriptor};
use tokio::time::Instant;
use tonic::transport::Channel;

use crate::discriptor;

use discriptor::discriptor::CachedDescriptors;

#[derive(Debug)]
pub struct ReflectionDiscriptorManager {
    pub cache: Arc<RwLock<CachedDescriptors>>,
    pub channel: Channel,
    pub refresh_interval: Duration,
    pub last_refresh: Arc<RwLock<Instant>>,
}

impl ReflectionDiscriptorManager {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let channel = tonic::transport::Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

        let manager = Self {
            cache: Arc::new(RwLock::new(CachedDescriptors::new())),
            channel,
            refresh_interval: Duration::from_secs(300), // 5 -min
            last_refresh: Arc::new(RwLock::new(Instant::now())),
        };

        // Ensure descriptors are loaded; propagate errors so callers see real cause
        manager.refresh_discriptors().await?;

        Ok(manager)
    }

    pub async fn refresh_discriptors(&self) -> Result<()> {
        // Build a fresh descriptor cache without holding locks across await
        let mut new_cache = CachedDescriptors::new();
        new_cache.load_discriptor(self.channel.clone()).await?;

        {
            let mut cache_guard = self.cache.write().unwrap();
            *cache_guard = new_cache;
        }

        // Debug: print discovered services to help diagnose mismatches
        {
            let cache = self.cache.read().unwrap();
            let services = cache.get_all_service();
            println!("[descriptor] loaded services: {:?}", services);
        }

        let mut refresh = self.last_refresh.write().unwrap();
        *refresh = Instant::now();
        Ok(())
    }

    // Get method descriptor from refresh discriptor
    pub async fn get_method(
        &self,
        service: &str,
        method: &str,
    ) -> Result<Option<MethodDescriptor>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.get_method(service, method).cloned())
    }

    pub async fn get_service(&self, service: &str) -> Option<ServiceDescriptor> {
        let cache = self.cache.read().unwrap();
        cache.get_service(service).cloned()
    }

    pub async fn list_services(&self) -> Vec<String> {
        let cache = self.cache.read().unwrap();
        cache.get_all_service()
    }

    pub async fn force_refreshd(&self) -> Result<()> {
        self.refresh_discriptors().await
    }
}

pub async fn get_discriptor_manager(endpoint: &str) -> Arc<ReflectionDiscriptorManager> {
    Arc::new(
        ReflectionDiscriptorManager::new(endpoint)
            .await
            .expect("msg"),
    )
}
