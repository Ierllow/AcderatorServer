use serde::Serialize;
use crate::common::ResponseHeader;
use crate::common::AppError;
use crate::score::{AuthUser, FromRef};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use sqlx::{MySqlPool, Row};

pub mod lib;

#[derive(Serialize)]
pub struct UserScore {
    pub score_id: i32,
    pub score: i32,
}

#[derive(Serialize)]
pub struct UserDataResponse {
    pub header: ResponseHeader,
    pub scores: Vec<UserScore>,
}

#[derive(Serialize)]
pub struct UserScores {
    pub scores: Vec<UserScore>,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserScores
where
    MySqlPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let pool = MySqlPool::from_ref(state);
        let rows = sqlx::query("SELECT score_id, score FROM score WHERE userid = ?")
            .bind(&auth_user.userid)
            .fetch_all(&pool)
            .await?;
        let scores = rows
            .iter()
            .map(|row| UserScore {
                score_id: row.get("score_id"),
                score: row.get("score"),
            })
            .collect();
        Ok(UserScores { scores })
    }
}

