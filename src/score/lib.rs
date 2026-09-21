use crate::auth::AuthUser;
use crate::common::{
    generate_token, hash_token, AppError, AppState, Msgpack, ResponseHeader,
    SCORE_SESSION_TTL_SECONDS,
};
use crate::query::{
    master::BaseScoreQuery,
    score::ScoreQuery,
    score_session::ScoreSessionQuery,
    song::{SongFilter, SongQuery},
};
use crate::score::*;
use axum::extract::State;

pub(super) async fn score_begin(
    State(state): State<AppState>,
    user: AuthUser,
    Msgpack(req): Msgpack<ScoreBeginRequest>,
) -> Result<Msgpack<ScoreBeginResponse>, AppError> {
    let song_exists = SongQuery::new(&state.pool)
        .filter(SongFilter::Sid(req.score_id))
        .any()
        .await?;

    if !song_exists {
        return Err(AppError::BadRequest("invalid request".into()));
    }

    let session_token = generate_token();
    let session_hash = hash_token(&session_token);
    let mut transaction = state.pool.begin().await?;
    ScoreSessionQuery::prune_expired(&mut transaction, SCORE_SESSION_TTL_SECONDS).await?;
    ScoreSessionQuery::invalidate_previous(&mut transaction, &user.userid, req.score_id).await?;
    ScoreSessionQuery::create(&mut transaction, &session_hash, req.score_id, &user.userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(ScoreBeginResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        session_id: session_token,
    }))
}

pub(super) async fn score_submit(
    State(state): State<AppState>,
    user: AuthUser,
    Msgpack(req): Msgpack<ScoreSubmitRequest>,
) -> Result<Msgpack<ScoreSubmitResponse>, AppError> {
    let base_score = BaseScoreQuery::new(&state.pool)
        .first()
        .await?
        .ok_or_else(|| AppError::ServiceFailure("base score not found".into()))?;
    if !(0..=base_score).contains(&req.score) {
        return Err(AppError::BadRequest("invalid request".into()));
    }

    if req.session_id.len() != 64 || !req.session_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::NotFound);
    }

    let session_hash = hash_token(&req.session_id);
    let mut transaction = state.pool.begin().await?;
    let score_session = ScoreSessionQuery::consume(
        &mut transaction,
        &session_hash,
        &user.userid,
        SCORE_SESSION_TTL_SECONDS,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    let score_id = score_session.score_id;
    ScoreQuery::upsert_best(&mut transaction, &user.userid, score_id, req.score).await?;

    transaction.commit().await?;

    Ok(Msgpack(ScoreSubmitResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
    }))
}
