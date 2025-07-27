use super::{
    config::{
        AUTH_RATE_LIMIT_MAX_REQUESTS, AUTH_RATE_LIMIT_WINDOW_SECONDS, MAINTENANCE_MODE,
        RATE_LIMIT_MAX_REQUESTS, RATE_LIMIT_WINDOW_SECONDS,
    },
    error::AppError,
    state::AppState,
};
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

pub async fn maintenance_guard(req: Request, next: Next) -> Result<Response, AppError> {
    if MAINTENANCE_MODE {
        return Err(AppError::Maintenance);
    }

    Ok(next.run(req).await)
}

pub async fn rate_limit_guard(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = connect_info
        .map(|ConnectInfo(addr)| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let path = req.uri().path();
    let (bucket, max_requests, window_seconds) = if path.starts_with("/auth/") {
        (
            "auth",
            AUTH_RATE_LIMIT_MAX_REQUESTS,
            AUTH_RATE_LIMIT_WINDOW_SECONDS,
        )
    } else {
        ("global", RATE_LIMIT_MAX_REQUESTS, RATE_LIMIT_WINDOW_SECONDS)
    };

    state
        .rate_limiter
        .check(format!("{bucket}:{key}"), max_requests, window_seconds)?;

    Ok(next.run(req).await)
}
