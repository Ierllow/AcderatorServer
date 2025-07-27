use serde::{Deserialize, Serialize};

pub mod lib;

pub struct AuthUser {
    pub userid: String,
}

#[derive(Deserialize)]
pub struct ScoreBeginRequest {
    score_id: String,
}

#[derive(Deserialize)]
pub struct ScoreSubmitRequest {
    session_id: String,
    score: i32,
}

#[derive(Serialize)]
pub struct ScoreBeginResponse {
    session_id: String,
}
