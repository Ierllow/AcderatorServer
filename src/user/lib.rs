use axum::{http::StatusCode, routing::get, Json, Router};
use sqlx::MySqlPool;
use crate::common::{AppError, ResponseHeader};
use crate::user::{UserDataResponse, UserScores};

pub fn routes() -> Router<MySqlPool> {
    Router::new().route("/:sid/user/data", get(user_data))
}

async fn user_data(scores: UserScores)
-> Result<Json<UserDataResponse>, AppError> {
    Ok(Json(UserDataResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
        scores: scores.scores,
    }))
}
