use crate::common::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    crate::routes! {
        get "/debug" => super::lib::debug_page,
        get "/debug/master" => super::lib::master_page,
        get "/debug/master/data" => super::lib::master_data,
        post "/debug/master/version" => super::lib::save_master_version,
        post "/debug/master/base-score" => super::lib::save_base_score,
        post "/debug/master/song" => super::lib::save_song,
    }
}
