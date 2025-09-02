use std::{collections::HashMap, sync::Mutex};

use super::model::ServiceConfig;
use crate::registery::model::InternalAuthConfig;
use crate::utils::model::ServiceRegisterRequest;
use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_MAP: Mutex<HashMap<String, ServiceConfig>> = Mutex::new(HashMap::new());
}

pub trait RegistryTrait {
    fn register(&self, req: ServiceRegisterRequest) -> Option<String>;
    fn discover(&self, service_name: String) -> Option<ServiceConfig>;
}

pub struct ServiceRegistry {}

impl RegistryTrait for ServiceRegistry {
    fn register(&self, req: ServiceRegisterRequest) -> Option<String> {
        let val = format!("http://{}:{}", req.host, req.port);
        let config = ServiceConfig {
            endpoint: val.to_string(),
            service_name: req.service_name.to_string(),
            auth_config: req
                .oauth_config
                .clone()
                .map(|x| InternalAuthConfig { auth_type: x }),
        };

        match GLOBAL_MAP.lock() {
            Ok(mut mp) => {
                mp.insert(req.service_name.to_string(), config);
                Some(val)
            }
            Err(_e) => None,
        }
    }

    fn discover(&self, service_name: String) -> Option<ServiceConfig> {
        match GLOBAL_MAP.lock() {
            Ok(mp) => mp.get(&service_name).cloned(),
            Err(_) => None,
        }
    }
}
