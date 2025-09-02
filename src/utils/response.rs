use reqwest::StatusCode;

#[derive(Debug)]
pub struct Response {
    pub message: String,
    pub status: String,
    pub status_code: StatusCode,
    pub data: Option<serde_json::Value>,
}
