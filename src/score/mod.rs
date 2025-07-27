use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use crate::common::*;

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
        let path_params: axum::extract::Path<std::collections::HashMap<String, String>> =
            axum::extract::Path::from_request_parts(parts, state).await
            .map_err(|_| AppError::BadRequest("URLにセッションIDが含まれていません".into()))?;

        let sid = path_params.get("sid").ok_or(AppError::Unauthorized)?;
        let pool = MySqlPool::from_ref(state);
        let row = sqlx::query("SELECT userid FROM session WHERE session_id = ? AND last_activity > NOW() - INTERVAL 60 MINUTE")
            .bind(&sid)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let userid: String = row.get("userid");

        sqlx::query("UPDATE session SET last_activity = NOW() WHERE session_id = ?")
            .bind(&sid)
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
