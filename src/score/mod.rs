use crate::common::ResponseHeader;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod lib;

#[derive(Deserialize)]
pub struct ScoreBeginRequest {
    score_id: i32,
}

#[derive(Deserialize)]
pub struct ScoreSubmitRequest {
    session_id: String,
    score: i32,
}

#[derive(Serialize)]
pub struct ScoreBeginResponse {
    pub header: ResponseHeader,
    session_id: String,
}

#[derive(Serialize)]
pub struct ScoreSubmitResponse {
    pub header: ResponseHeader,
}
