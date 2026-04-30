use crate::common::{AppError, AppState, Msgpack, ResponseHeader};
use crate::query::{
    master::BaseScoreQuery,
    score::ScoreQuery,
    score_session::{ScoreSessionFilter, ScoreSessionQuery},
    song::{SongFilter, SongQuery},
};
use crate::score::*;
use axum::extract::State;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

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

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut transaction = state.pool.begin().await?;

    ScoreSessionQuery::create(&mut transaction, &session_id, req.score_id, &user.userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(ScoreBeginResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        session_id,
    }))
}

pub(super) async fn score_submit(
    State(state): State<AppState>,
    user: AuthUser,
    Msgpack(req): Msgpack<ScoreSubmitRequest>,
) -> Result<Msgpack<ScoreSubmitResponse>, AppError> {
    let mut transaction = state.pool.begin().await?;

    let score_session = ScoreSessionQuery::new(&state.pool)
        .filter(ScoreSessionFilter::SessionId(&req.session_id))
        .filter(ScoreSessionFilter::ActiveWithinMinutes(60))
        .first()
        .await?
        .ok_or(AppError::NotFound)?;

    if score_session.userid != user.userid {
        return Err(AppError::Forbidden);
    }

    let score_id = score_session.score_id;
    let base_score = BaseScoreQuery::new(&state.pool)
        .first()
        .await?
        .ok_or_else(|| AppError::ServiceFailure("base score not found".into()))?;
    if req.score > base_score {
        return Err(AppError::BadRequest("invalid request".into()));
    }

    ScoreQuery::upsert_best(&mut transaction, &user.userid, score_id, req.score).await?;
    ScoreSessionQuery::delete_by_session_id(&mut transaction, &req.session_id).await?;

    transaction.commit().await?;

    Ok(Msgpack(ScoreSubmitResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
    }))
}
