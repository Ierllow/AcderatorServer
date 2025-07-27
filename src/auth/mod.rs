use serde::{Deserialize, Serialize};
use crate::common::ResponseHeader;

pub mod lib;

#[derive(Deserialize)]
pub struct AuthRequest {
    userid: String,
    password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub header: ResponseHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}
