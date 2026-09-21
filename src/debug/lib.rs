use super::{
    DebugSong, MasterCounts, MasterDebugData, MasterSaveResponse, SaveBaseScoreRequest,
    SaveMasterVersionRequest, SaveSongRequest,
};
use crate::common::{AppError, AppState};
use crate::query::master::{BaseScoreQuery, MasterDataQuery, MasterVersionQuery};
use crate::query::song::{SongQuery, SongUpsert};
use axum::{
    extract::State,
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

const DEBUG_HTML: &str = include_str!("debug.html");
const MASTER_HTML: &str = include_str!("master.html");
const ADMIN_CSS: &str = include_str!("admin.css");

pub(super) async fn debug_page() -> Result<Html<&'static str>, AppError> {
    Ok(Html(DEBUG_HTML))
}

pub(super) async fn master_page() -> Result<Html<&'static str>, AppError> {
    Ok(Html(MASTER_HTML))
}

pub(super) async fn admin_css() -> Response {
    let mut response = ADMIN_CSS.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    response
}

pub(super) async fn master_data(
    State(state): State<AppState>,
) -> Result<Json<MasterDebugData>, AppError> {
    let (version, base_score, songs, counts) = tokio::try_join!(
        MasterVersionQuery::new(&state.pool).first(),
        BaseScoreQuery::new(&state.pool).first(),
        SongQuery::new(&state.pool).all(),
        MasterDataQuery::counts(&state.pool),
    )?;
    let version = version.unwrap_or_default();
    let song_rows: Vec<DebugSong> = songs.into_iter().map(DebugSong::from).collect();
    let base_score_rows = base_score
        .map(|score| vec![json!({ "score": score })])
        .unwrap_or_default();
    let raw = json!({
        "version_master": version.clone(),
        "song_masters": &song_rows,
        "base_score_masters": &base_score_rows
    });

    Ok(Json(MasterDebugData {
        version,
        base_score,
        counts: MasterCounts::from(counts),
        raw,
    }))
}

pub(super) async fn save_master_version(
    State(state): State<AppState>,
    Json(req): Json<SaveMasterVersionRequest>,
) -> Result<Json<MasterSaveResponse>, AppError> {
    MasterVersionQuery::replace(&state.pool, &req.version).await?;

    Ok(Json(MasterSaveResponse {
        ok: true,
        message: "master_version updated in database only",
    }))
}

pub(super) async fn save_base_score(
    State(state): State<AppState>,
    Json(req): Json<SaveBaseScoreRequest>,
) -> Result<Json<MasterSaveResponse>, AppError> {
    BaseScoreQuery::replace(&state.pool, req.score).await?;

    Ok(Json(MasterSaveResponse {
        ok: true,
        message: "base_score updated in database only",
    }))
}

pub(super) async fn save_song(
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
