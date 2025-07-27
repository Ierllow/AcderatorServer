use axum::{extract::State, http::StatusCode, Json, Router, routing::post};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;
use crate::auth::*;
use crate::master::MasterDataResponse;
use crate::common::{AppError, CustomHeader};

pub fn routes() -> Router<MySqlPool> {
    Router::new().route("/auth/login", post(login))
}

async fn login(
    State(pool): State<MySqlPool>,
    headers: axum::http::HeaderMap,
    Json(request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
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

    let mut transaction= pool.begin().await?;
    let user = sqlx::query("SELECT * FROM user WHERE userid = ?")
        .bind(&request.userid)
        .fetch_optional(&mut *transaction)
        .await?;

    if let Some(row) = user {
        let db_password: String = row.get("password");
        if db_password != request.password {
            return Err(AppError::Unauthorized);
        }
    } else {
        sqlx::query("INSERT INTO user (userid, password) VALUES (?, ?)")
            .bind(&request.userid)
            .bind(&request.password)
            .execute(&pool)
            .await?;
    }

    let session_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO session (session_id, userid, last_activity) VALUES (?, ?, NOW())")
        .bind(&session_id)
        .bind(&request.userid)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(Json(AuthResponse {
        header: ResponseHeader {
            status: StatusCode::OK.as_u16(),
            master: None,
        },
        token: Some(session_id),
    }))
}
