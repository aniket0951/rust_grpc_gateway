use super::auth::Auth;

#[derive(Debug, Clone)]
pub struct APIKeyAuth {
    pub header_name: String,
    pub value: String,
}

impl Auth for APIKeyAuth {
    fn header_name(&self) -> &str {
        &self.header_name
    }

    fn value(&self) -> String {
        self.value.to_string()
    }
}
