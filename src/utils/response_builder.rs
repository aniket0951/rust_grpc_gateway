use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBuilder<T>
where
    T: Serialize,
{
    pub status: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ResponseBuilder<T>
where
    T: Serialize,
{
    pub fn success(msg: String, data: T) -> Self {
        Self {
            status: true,
            message: msg,
            data: Some(data),
        }
    }

    pub fn bad_request(msg: String) -> Self {
        Self {
            status: false,
            message: msg,
            data: None,
        }
    }
}

impl<T> Display for ResponseBuilder<T>
where
    T: Serialize + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json_string = serde_json::to_string(self).map_err(|_| fmt::Error)?;

        write!(f, "{}", json_string)
    }
}
