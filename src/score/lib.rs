use axum::{async_trait, extract::{FromRequestParts, State}, http::StatusCode, routing::post, Json, Router, http::request::Parts};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{MySqlPool, Row};
use crate::score::{ScoreBeginRequest, ScoreBeginResponse, ScoreSubmitRequest, AuthUser};
use crate::utils::AppError;

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/:sid/score/begin", post(score_begin))
        .route("/:sid/score/submit", post(score_submit))
}

async fn score_begin(
    State(pool): State<MySqlPool>,
    user: AuthUser,
    Json(request): Json<ScoreBeginRequest>,
) -> Result<Json<ScoreBeginResponse>, AppError> {
    if request.score_id.is_empty() {
        return Err(AppError::BadRequest("score_id is required".into()));
    }

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO score_sessions (session_id, score_id, userid) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(&request.score_id)
        .bind(&user.userid)
        .execute(&pool)
        .await?;

    Ok(Json(ScoreBeginResponse { session_id: session_id }))
}

async fn score_submit(
    State(pool): State<MySqlPool>,
    user: AuthUser,
    Json(request): Json<ScoreSubmitRequest>,
) -> Result<StatusCode, AppError> {
    let row = sqlx::query("SELECT score_id FROM score_sessions WHERE session_id = ? AND userid = ?")
        .bind(&request.session_id)
        .bind(&user.userid)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let score_id: String = row.get("score_id");

    sqlx::query("INSERT INTO score (userid, score_id, score) VALUES (?, ?, ?)")
        .bind(&user.userid)
        .bind(&score_id)
        .bind(request.score)
        .execute(&pool)
        .await?;

    Ok(StatusCode::OK)
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    MySqlPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let sid = parts.uri.path()
            .split('/')
            .nth(1)
            .ok_or(AppError::Unauthorized)?;

        let pool = MySqlPool::from_ref(state);

        let row = sqlx::query("SELECT userid FROM sessions WHERE session_id = ? AND last_activity > NOW() - INTERVAL 60 MINUTE")
            .bind(sid)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let userid: String = row.get("userid");

        sqlx::query("UPDATE sessions SET last_activity = NOW() WHERE session_id = ?")
            .bind(sid)
            .execute(&pool)
            .await?;

        Ok(AuthUser { userid })
    }
}

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<MySqlPool> for MySqlPool {
    fn from_ref(input: &MySqlPool) -> Self {
        input.clone()
    }
}
