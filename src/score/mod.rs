use crate::common::{AppError, AppState, ResponseHeader};
use crate::query::session::{SessionFilter, SessionQuery};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

pub mod api;
pub mod lib;

pub struct AuthUser {
    pub userid: String,
}

#[derive(Deserialize)]
pub struct ScoreBeginRequest {
    score_id: i32,
}

#[derive(Deserialize)]
pub struct ScoreSubmitRequest {
    session_id: String,
    score: i32,
}

#[derive(Serialize)]
pub struct ScoreBeginResponse {
    pub header: ResponseHeader,
    session_id: String,
}

#[derive(Serialize)]
pub struct ScoreSubmitResponse {
    pub header: ResponseHeader,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    MySqlPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let sid = authorization
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let pool = MySqlPool::from_ref(state);

        let session = SessionQuery::new(&pool)
            .filter(SessionFilter::SessionId(sid))
            .filter(SessionFilter::ActiveWithinMinutes(60))
            .first()
            .await?
            .ok_or(AppError::Unauthorized)?;

        SessionQuery::touch(&pool, sid).await?;

        Ok(AuthUser {
            userid: session.userid,
        })
    }
}

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl FromRef<AppState> for MySqlPool {
    fn from_ref(input: &AppState) -> Self {
        input.pool.clone()
    }
}
