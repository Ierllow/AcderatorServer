use crate::common::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    crate::routes! {
        get "/user/data" => super::lib::user_data,
    }
}
