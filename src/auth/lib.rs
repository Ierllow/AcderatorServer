use rand::{Rng, thread_rng};
use axum::http::HeaderMap;
use axum::{extract::State, http::StatusCode, Json, Router, routing::post};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;
use crate::auth::*;
use crate::master::MasterDataResponse;
use crate::common::{AppError, CustomHeader};

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
}

async fn login(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {

    let row = sqlx::query("SELECT version FROM master_version LIMIT 1")
        .fetch_one(&pool)
        .await?;
    let current_version: String = row.get("version");
    let header_raw = headers
        .get("header")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("ヘッダーがありません".into()))?;
    let custom_header: CustomHeader = serde_json::from_str(header_raw)
        .map_err(|_| AppError::BadRequest("ヘッダーの形式が不正です".into()))?;

    if custom_header.master != current_version {
        let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
        let data = std::fs::read_to_string(&path)?;
        let master_data: MasterDataResponse = serde_json::from_str(&data)?;
        return Err(AppError::PreconditionFailed(Some(master_data)));
    }

    let mut transaction = pool.begin().await?;
    let user = sqlx::query("SELECT * FROM user WHERE userid = ?")
        .bind(&request.userid)
        .fetch_optional(&mut *transaction)
        .await?;

    let row = match user {
        Some(row) => row,
        None => return Err(AppError::Unauthorized),
    };
    let db_password: String = row.get("password");
    if db_password != request.password {
        return Err(AppError::Unauthorized);
    }

    let session_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO session (session_id, userid, last_activity) VALUES (?, ?, NOW())")
        .bind(&session_id)
        .bind(&request.userid)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(Json(LoginResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
        token: Some(session_id),
    }))
}

async fn register(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    let row = sqlx::query("SELECT version FROM master_version LIMIT 1")
        .fetch_one(&pool)
        .await?;
    let current_version: String = row.get("version");

    let header_raw = headers
        .get("header")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::BadRequest("ヘッダーがありません".into()))?;
    let custom_header: CustomHeader = serde_json::from_str(header_raw)
        .map_err(|_| AppError::BadRequest("ヘッダーの形式が不正です".into()))?;

    if custom_header.master != current_version {
        let path = std::env::var("MASTER_DATA_PATH").expect("MASTER_DATA_PATH must be set");
        let data = std::fs::read_to_string(&path)?;
        let master_data: MasterDataResponse = serde_json::from_str(&data)?;
        return Err(AppError::PreconditionFailed(Some(master_data)));
    }

    let mut transaction = pool.begin().await?;

    let userid: u32 = {
        let mut rng = thread_rng();
        rng.gen_range(100_000_000..1_000_000_000)
    };

    let existing = sqlx::query("SELECT 1 FROM user WHERE uuid = ?")
        .bind(&request.uuid)
        .fetch_optional(&mut *transaction)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("生成したユーザーIDが重複しました".into()));
    }

    sqlx::query("INSERT INTO user (userid, uuid, password) VALUES (?, ?, ?)")
        .bind(&userid)
        .bind(&request.uuid)
        .bind(&userid)
        .execute(&mut *transaction)
        .await?;

    let session_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO session (session_id, userid, last_activity) VALUES (?, ?, NOW())")
        .bind(&session_id)
        .bind(&userid)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(Json(RegisterResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
        token: Some(session_id),
        userid: userid,
        password: userid.to_string(),
    }))
}
