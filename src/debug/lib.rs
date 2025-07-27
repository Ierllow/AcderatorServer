use super::{
    DebugSong, MasterCounts, MasterDebugData, MasterSaveResponse, SaveBaseScoreRequest,
    SaveMasterVersionRequest, SaveSongRequest,
};
use crate::common::{AppError, AppState};
use crate::query::master::{BaseScoreQuery, MasterVersionQuery};
use crate::query::song::{SongQuery, SongUpsert};
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

const DEBUG_HTML: &str = include_str!("debug.html");
const MASTER_HTML: &str = include_str!("master.html");

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/debug", get(debug_page))
        .route("/debug/master", get(master_page))
        .route("/debug/master/data", get(master_data))
        .route("/debug/master/version", post(save_master_version))
        .route("/debug/master/base-score", post(save_base_score))
        .route("/debug/master/song", post(save_song))
}

async fn debug_page() -> Result<Html<&'static str>, AppError> {
    Ok(Html(DEBUG_HTML))
}

async fn master_page() -> Result<Html<&'static str>, AppError> {
    Ok(Html(MASTER_HTML))
}

async fn master_data(State(state): State<AppState>) -> Result<Json<MasterDebugData>, AppError> {
    let version = MasterVersionQuery::new(&state.pool)
        .first()
        .await?
        .unwrap_or_default();
    let base_score = BaseScoreQuery::new(&state.pool).first().await?;
    let songs = SongQuery::new(&state.pool).all().await?;
    let song_rows: Vec<DebugSong> = songs.into_iter().map(DebugSong::from).collect();
    let base_score_rows = base_score
        .map(|score| vec![json!({ "score": score })])
        .unwrap_or_default();
    let song_count = song_rows.len();
    let base_score_count = base_score_rows.len();
    let raw = json!({
        "version_master": version.clone(),
        "song_masters": song_rows,
        "base_score_masters": base_score_rows
    });

    Ok(Json(MasterDebugData {
        version,
        base_score,
        counts: MasterCounts {
            master_version: if raw["version_master"]
                .as_str()
                .unwrap_or_default()
                .is_empty()
            {
                0
            } else {
                1
            },
            title_masters: 0,
            song_select_masters: 0,
            song_masters: song_count,
            score_rate_masters: 0,
            base_score_masters: base_score_count,
            judge_zone_masters: 0,
            base_hp_masters: 0,
            hp_rate_masters: 0,
            sound_sheet_masters: 0,
            result_masters: 0,
        },
        raw,
    }))
}

async fn save_master_version(
    State(state): State<AppState>,
    Json(req): Json<SaveMasterVersionRequest>,
) -> Result<Json<MasterSaveResponse>, AppError> {
    MasterVersionQuery::replace(&state.pool, &req.version).await?;

    Ok(Json(MasterSaveResponse {
        ok: true,
        message: "master_version updated in database only",
    }))
}

async fn save_base_score(
    State(state): State<AppState>,
    Json(req): Json<SaveBaseScoreRequest>,
) -> Result<Json<MasterSaveResponse>, AppError> {
    BaseScoreQuery::replace(&state.pool, req.score).await?;

    Ok(Json(MasterSaveResponse {
        ok: true,
        message: "base_score updated in database only",
    }))
}

async fn save_song(
    State(state): State<AppState>,
    Json(req): Json<SaveSongRequest>,
) -> Result<Json<MasterSaveResponse>, AppError> {
    SongQuery::upsert(
        &state.pool,
        SongUpsert {
            sid: req.sid,
            group: req.group,
            difficulty: req.difficulty,
            name: req.name,
            composer: req.composer,
            start_offset: req.start_offset,
            bg: req.bg,
        },
    )
    .await?;

    Ok(Json(MasterSaveResponse {
        ok: true,
        message: "song updated in database only",
    }))
}
