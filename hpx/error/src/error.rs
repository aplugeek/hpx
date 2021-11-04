use hyper::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppResponseError {
    #[serde(rename = "code")]
    pub code: u16,
    #[serde(rename = "message")]
    pub message: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub status_code: StatusCode,
}

impl AppResponseError {
    pub fn from(code: u16, s: &str, status_code: StatusCode) -> Self {
        AppResponseError {
            code,
            message: s.into(),
            status_code,
        }
    }
}
