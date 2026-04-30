use crate::common::AppError;
use crate::common::ResponseHeader;
use crate::query::score::{ScoreFilter, ScoreQuery};
use crate::score::{AuthUser, FromRef};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::Serialize;
use sqlx::MySqlPool;

pub mod api;
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
        let scores = ScoreQuery::new(&pool)
            .filter(ScoreFilter::Userid(&auth_user.userid))
            .all()
            .await?
            .into_iter()
            .map(|score| UserScore {
                score_id: score.score_id,
                score: score.score,
            })
            .collect();
        Ok(UserScores { scores })
    }
}
