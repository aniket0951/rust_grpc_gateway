use std::sync::Mutex;
use std::time::Instant;

use lazy_static::lazy_static;
use std::collections::HashMap;
lazy_static! {
    static ref token_orchestrator_map: Mutex<HashMap<String, TokenOrchestrator>> =
        Mutex::new(HashMap::new());
}

pub struct TokenOrchestrator {
    pub service_name: String,
    pub token_config: TokenConfig,
}

pub struct TokenConfig {
    pub token: String,
    pub expired_at: Instant,
    pub is_refresh_success: bool,
}
