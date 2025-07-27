use crate::common::{AppError, AppState, Msgpack, ResponseHeader};
use crate::user::{UserDataResponse, UserScores};
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/user/data", get(user_data))
}

async fn user_data(scores: UserScores) -> Result<Msgpack<UserDataResponse>, AppError> {
    Ok(Msgpack(UserDataResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        scores: scores.scores,
    }))
}
