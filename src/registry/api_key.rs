use super::auth::Auth;

#[derive(Debug, Clone)]
pub struct APIKeyAuth {
    pub header_name: String,
    pub value: String,
}

impl APIKeyAuth {
    pub fn new(header_name: String, value: String) -> Self {
        Self { header_name, value }
    }
}

impl Auth for APIKeyAuth {
    fn header_name(&self) -> &str {
        &self.header_name
    }

    fn value(&self) -> String {
        self.value.to_string()
    }
}
