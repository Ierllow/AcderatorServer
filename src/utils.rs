use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

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
    PreconditionFailed,
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
        let status = match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            AppError::JsonError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        match self {
            AppError::Internal(ref e) => eprintln!("[Critical] {:?}", e),
            AppError::SqlError(ref e) => eprintln!("[Database] {:?}", e),
            _ => eprintln!("[Info] クライアントエラー: {}", self),
        }
        let body = Json(json!({
            "error": {
                "status": status.as_u16(),
                "message": self.to_string()
            }
        }));

        (status, body).into_response()
    }
}
