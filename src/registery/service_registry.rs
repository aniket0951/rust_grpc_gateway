use std::{collections::HashMap, sync::Mutex};

use crate::utils::model::ServiceRegisterRequest;
use lazy_static::lazy_static;

use crate::utils::service_status::ServiceStatus;

lazy_static! {
    static ref GLOBAL_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref unavailable_service_map: Mutex<HashMap<String, ServiceStatus>> =
        Mutex::new(HashMap::new());
}

pub trait RegistryTrait {
    fn register(&self, req: ServiceRegisterRequest) -> Option<String>;
    fn discover(&self, service_name: String) -> Option<String>;
}

pub struct ServiceRegistry {}

impl RegistryTrait for ServiceRegistry {
    fn register(&self, req: ServiceRegisterRequest) -> Option<String> {
        let val = format!("http://{}:{}", req.host, req.port);

        match GLOBAL_MAP.lock() {
            Ok(mut mp) => {
                mp.insert(req.service_name, val.to_string());
                Some(val)
            }
            Err(_e) => None,
        }
    }

    fn discover(&self, service_name: String) -> Option<String> {
        match GLOBAL_MAP.lock() {
            Ok(mp) => mp.get(&service_name).cloned(),
            Err(_) => None,
        }
    }
}
