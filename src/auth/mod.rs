use serde::{Deserialize, Serialize};

pub mod lib;

#[derive(Deserialize)]
pub struct AuthRequest {
    userid: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
}
