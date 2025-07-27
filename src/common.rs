use serde::{Deserialize, Serialize};
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;
use sqlx::{MySql, QueryBuilder};
use crate::master::MasterDataResponse;

#[derive(Serialize)]
pub struct ResponseHeader {
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<MasterDataResponse>,
}

#[derive(Deserialize)]
pub struct CustomHeader {
    pub master: String,
    pub token: Option<String>,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("リクエストが正しくありません: {0}")]
    BadRequest(String),
    #[error("ログインが必要です")]
    Unauthorized,
    #[error("アクセス権限がありません")]
    Forbidden,
    #[error("指定されたリソースが見つかりません")]
    NotFound,
    #[error("既に登録されています")]
    Conflict,
    #[error("データが変更されています。再度お試しください")]
    PreconditionFailed(Option<MasterDataResponse>),
    #[error("サーバー内部でエラーが発生しました")]
    Internal(#[from] anyhow::Error),
    #[error("データベースエラー")]
    SqlError(#[from] sqlx::Error),
    #[error("ファイル処理エラー")]
    IoError(#[from] std::io::Error),
    #[error("データ形式エラー")]
    JsonError(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_message = self.to_string();
        let mut master_data = None;
        let status = match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::PreconditionFailed(m) => {
                master_data = m;
                StatusCode::PRECONDITION_FAILED
            }
            AppError::JsonError(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(e) => {
                eprintln!("[Critical] {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::SqlError(e) => {
                eprintln!("[Database] {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::IoError(e) => {
                eprintln!("[IO] {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = Json(json!({
            "header": {
                "status": status.as_u16(),
                "master": master_data,
            },
            "error": error_message
        }));

        (status, body).into_response()
    }
}

pub async fn bulk_insert<'a, T, F>(
    transaction: &mut sqlx::Transaction<'_, MySql>,
    prefix: &str,
    items: &'a [T],
    mut binder: F,
    suffix: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: 'a,
    F: FnMut(sqlx::query_builder::Separated<'_, 'a, MySql, &str>, &'a T),
{
    if items.is_empty() {
        return Ok(());
    }

    let mut qb = QueryBuilder::new(prefix);
    qb.push_values(items, |separated, item| {
        binder(separated, item);
    });

    if !suffix.is_empty() {
        qb.push(" ");
        qb.push(suffix);
    }

    qb.build().execute(&mut **transaction).await?;
    Ok(())
}
