use crate::common::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    crate::routes! {
        post "/auth/login" => super::lib::login,
        post "/auth/register" => super::lib::register,
    }
}
