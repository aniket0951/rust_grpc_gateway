use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ValidationError(pub String);

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ValidationError {}
