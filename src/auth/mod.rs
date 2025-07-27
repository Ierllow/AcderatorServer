use serde::{Deserialize, Serialize};
use crate::common::ResponseHeader;

pub mod lib;

#[derive(Deserialize)]
pub struct LoginRequest {
    userid: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    uuid: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub header: ResponseHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub header: ResponseHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub userid: u32,
    pub password: String,
}