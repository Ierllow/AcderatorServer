use crate::auth::*;
use crate::common::{
    generate_token, hash_password, hash_token, verify_password, AppError, AppState, CustomHeader,
    Msgpack, PasswordMatch, ResponseHeader, AUTH_SESSION_TTL_SECONDS, MAX_PASSWORD_BYTES,
    MAX_UUID_BYTES,
};
use crate::master::MasterDataResponse;
use crate::query::{
    master::MasterVersionQuery,
    session::SessionQuery,
    user::{UserFilter, UserQuery},
};
use axum::{extract::State, http::HeaderMap};
use rand::{rngs::OsRng, Rng};
use sqlx::MySqlPool;

pub(super) async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Msgpack(req): Msgpack<LoginRequest>,
) -> Result<Msgpack<LoginResponse>, AppError> {
    validate_master_version(&state.pool, &headers).await?;
    if req.userid.is_empty() || req.userid.len() > 255 || req.password.len() > MAX_PASSWORD_BYTES {
        return Err(AppError::Unauthorized);
    }

    let user = UserQuery::new(&state.pool)
        .filter(UserFilter::Userid(&req.userid))
        .first()
        .await?
        .ok_or(AppError::Unauthorized)?;

    let password = req.password;
    let password_match = verify_password_async(user.password, password.clone()).await?;
    if password_match == PasswordMatch::Invalid {
        return Err(AppError::Unauthorized);
    }

    let replacement_hash = if password_match == PasswordMatch::Legacy {
        Some(hash_password_async(password).await?)
    } else {
        None
    };

    let session_token = generate_token();
    let session_hash = hash_token(&session_token);
    let mut transaction = state.pool.begin().await?;
    if let Some(replacement_hash) = replacement_hash {
        UserQuery::update_password(&mut transaction, &req.userid, &replacement_hash).await?;
    }
    SessionQuery::prune_expired(&mut transaction, AUTH_SESSION_TTL_SECONDS).await?;
    SessionQuery::create(&mut transaction, &session_hash, &req.userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(LoginResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        token: Some(session_token),
    }))
}

pub(super) async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Msgpack(req): Msgpack<RegisterRequest>,
) -> Result<Msgpack<RegisterResponse>, AppError> {
    validate_master_version(&state.pool, &headers).await?;
    if req.uuid.trim().is_empty() || req.uuid.len() > MAX_UUID_BYTES {
        return Err(AppError::BadRequest("invalid uuid".into()));
    }

    let userid: u32 = {
        let mut rng = OsRng;
        rng.gen_range(100_000_000..1_000_000_000)
    };

    let existing = UserQuery::new(&state.pool)
        .filter(UserFilter::Uuid(&req.uuid))
        .any()
        .await?;
    if existing {
        return Err(AppError::Conflict);
    }

    let password_hash = hash_password_async(userid.to_string()).await?;
    let session_token = generate_token();
    let session_hash = hash_token(&session_token);
    let mut transaction = state.pool.begin().await?;

    UserQuery::create(&mut transaction, userid, &req.uuid, &password_hash).await?;
    SessionQuery::prune_expired(&mut transaction, AUTH_SESSION_TTL_SECONDS).await?;
    SessionQuery::create(&mut transaction, &session_hash, userid).await?;

    transaction.commit().await?;

    Ok(Msgpack(RegisterResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
        token: Some(session_token),
        userid,
        password: "".into(),
    }))
}

pub(super) async fn logout(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Msgpack<LogoutResponse>, AppError> {
    let mut transaction = state.pool.begin().await?;
    SessionQuery::delete(&mut transaction, &user.session_hash).await?;
    transaction.commit().await?;

    Ok(Msgpack(LogoutResponse {
        header: ResponseHeader {
            code: 0,
            master: None,
        },
    }))
}

async fn validate_master_version(pool: &MySqlPool, headers: &HeaderMap) -> Result<(), AppError> {
    let current_version = MasterVersionQuery::new(pool)
        .first()
        .await?
        .ok_or_else(|| AppError::ServiceFailure("master version not found".into()))?;
    let header_raw = headers
        .get("header")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("invalid request".into()))?;
    let custom_header: CustomHeader = serde_json::from_str(header_raw)
        .map_err(|_| AppError::BadRequest("invalid request".into()))?;

    if custom_header.master == current_version {
        return Ok(());
    }

    let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
    let data = std::fs::read_to_string(path)?;
    let master_data: MasterDataResponse = serde_json::from_str(&data)?;
    Err(AppError::PreconditionFailed(Box::new(Some(master_data))))
}

async fn hash_password_async(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|error| AppError::ServiceFailure(format!("password task failed: {error}")))?
}

async fn verify_password_async(
    stored: String,
    candidate: String,
) -> Result<PasswordMatch, AppError> {
    tokio::task::spawn_blocking(move || verify_password(&stored, &candidate))
        .await
        .map_err(|error| AppError::ServiceFailure(format!("password task failed: {error}")))
}
