use crate::common::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    crate::routes! {
        post "/score/begin" => super::lib::score_begin,
        post "/score/submit" => super::lib::score_submit,
    }
}
