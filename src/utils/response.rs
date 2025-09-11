use std::borrow::Cow;

use reqwest::StatusCode;

// #[derive(Debug)]
// pub struct Response {
//     pub message: String,
//     pub status: String,
//     pub status_code: StatusCode,
//     pub data: Option<serde_json::Value>,
// }
//
#[derive(Debug)]
pub struct Response {
    pub message: Cow<'static, str>,
    pub status: Cow<'static, str>,
    pub status_code: StatusCode,
    pub data: Option<serde_json::Value>,
}
