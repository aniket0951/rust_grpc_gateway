
#[derive(Debug, Clone)]
pub enum AuthType {
    APIKey { header_name: String, value: String },
    JWTToken { header_name: String, value: String },
}
#[derive(Debug, Clone)]
pub struct InternalAuthConfig {
    pub auth_type: AuthType,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub endpoint: String,
    pub service_name: String,
    pub auth_config: Option<InternalAuthConfig>,
}
