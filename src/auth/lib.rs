use crate::auth::*;
use crate::common::{AppError, AppState, CustomHeader, Msgpack, ResponseHeader};
use crate::master::MasterDataResponse;
use crate::query::{
    master::MasterVersionQuery,
    session::SessionQuery,
    user::{UserFilter, UserQuery},
};
use axum::http::HeaderMap;
use axum::{extract::State, routing::post, Router};
use rand::{thread_rng, Rng};
use uuid::Uuid;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Msgpack(req): Msgpack<LoginRequest>,
) -> Result<Msgpack<LoginResponse>, AppError> {
    let current_version = MasterVersionQuery::new(&state.pool)
        .first()
        .await?
        .ok_or_else(|| AppError::ServiceFailure("master version not found".into()))?;

    let header_raw = headers
        .get("header")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("invalid request".into()))?;
    let custom_header: CustomHeader = serde_json::from_str(header_raw)
        .map_err(|_| AppError::BadRequest("invalid request".into()))?;

    if custom_header.master != current_version {
        let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
        let data = std::fs::read_to_string(&path)?;
        let master_data: MasterDataResponse = serde_json::from_str(&data)?;
        return Err(AppError::PreconditionFailed(Box::new(Some(master_data))));
    }

    let mut transaction = state.pool.begin().await?;
    let user = UserQuery::new(&state.pool)
        .filter(UserFilter::Userid(&req.userid))
        .first()
        .await?;

    let user = match user {
        Some(user) => user,
        None => return Err(AppError::Unauthorized),
    };

    if user.password != req.password {
        return Err(AppError::Unauthorized);
    }

    let session_id = Uuid::new_v4().to_string();
    SessionQuery::create(&mut transaction, &session_id, &req.userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(LoginResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        token: Some(session_id),
    }))
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Msgpack(req): Msgpack<RegisterRequest>,
) -> Result<Msgpack<RegisterResponse>, AppError> {
    let current_version = MasterVersionQuery::new(&state.pool)
        .first()
        .await?
        .ok_or_else(|| AppError::ServiceFailure("master version not found".into()))?;

    let header_raw = headers
        .get("header")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("invalid request".into()))?;
    let custom_header: CustomHeader = serde_json::from_str(header_raw)
        .map_err(|_| AppError::BadRequest("invalid request".into()))?;

    if custom_header.master != current_version {
        let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
        let data = std::fs::read_to_string(&path)?;
        let master_data: MasterDataResponse = serde_json::from_str(&data)?;
        return Err(AppError::PreconditionFailed(Box::new(Some(master_data))));
    }

    let mut transaction = state.pool.begin().await?;

    let userid: u32 = {
        let mut rng = thread_rng();
        rng.gen_range(100_000_000..1_000_000_000)
    };

    let existing = UserQuery::new(&state.pool)
        .filter(UserFilter::Uuid(&req.uuid))
        .any()
        .await?;
    if existing {
        return Err(AppError::Conflict);
    }

    UserQuery::create(&mut transaction, userid, &req.uuid, userid).await?;

    let session_id = Uuid::new_v4().to_string();
    SessionQuery::create(&mut transaction, &session_id, userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(RegisterResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        token: Some(session_id),
        userid,
        password: "".into(),
    }))
}
