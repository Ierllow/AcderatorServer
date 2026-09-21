use crate::common::{hash_token, AppError, FromRef, ResponseHeader, AUTH_SESSION_TTL_SECONDS};
use crate::query::session::SessionQuery;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

pub mod api;
pub mod lib;

#[derive(Deserialize)]
pub struct LoginRequest {
    userid: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    uuid: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub header: ResponseHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub header: ResponseHeader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub userid: u32,
    pub password: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub header: ResponseHeader,
}

pub struct AuthUser {
    pub userid: String,
    pub(crate) session_hash: String,
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
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let (scheme, token) = authorization
            .split_once(' ')
            .ok_or(AppError::Unauthorized)?;
        if !scheme.eq_ignore_ascii_case("Bearer")
            || token.len() != 64
            || !token.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(AppError::Unauthorized);
        }

        let pool = MySqlPool::from_ref(state);
        let session_hash = hash_token(token);
        let session = SessionQuery::authenticate(&pool, &session_hash, AUTH_SESSION_TTL_SECONDS)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser {
            userid: session.userid,
            session_hash,
        })
    }
}
