use super::codec::Msgpack;
use crate::master::MasterDataResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize)]
pub struct ResponseHeader {
    pub code: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<MasterDataResponse>,
}

#[derive(Deserialize)]
pub struct CustomHeader {
    pub master: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("bad request")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("precondition failed")]
    PreconditionFailed(Box<Option<MasterDataResponse>>),
    #[error("maintenance mode")]
    Maintenance,
    #[error("unsupported media type")]
    UnsupportedMediaType,
    #[error("too many requests")]
    TooManyRequests,
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("invalid request format")]
    InvalidRequestFormat(String),
    #[error("response failed")]
    ResponseFailure(String),
    #[error("data operation failed")]
    DataFailure(String),
    #[error("resource operation failed")]
    ResourceFailure(String),
    #[error("service failed")]
    ServiceFailure(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    header: ResponseHeader,
    error: &'static str,
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::ServiceFailure(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DataFailure(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::ResourceFailure(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InvalidRequestFormat(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for AppError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        AppError::InvalidRequestFormat(err.to_string())
    }
}

impl From<rmp_serde::encode::Error> for AppError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        AppError::ResponseFailure(err.to_string())
    }
}

impl AppError {
    fn app_code(&self) -> u32 {
        match self {
            AppError::BadRequest(_) => 10001,
            AppError::InvalidRequestFormat(_) => 10002,
            AppError::Unauthorized => 11001,
            AppError::Forbidden => 11002,
            AppError::NotFound => 12001,
            AppError::Conflict => 12002,
            AppError::PreconditionFailed(_) => 13001,
            AppError::Maintenance => 13002,
            AppError::UnsupportedMediaType => 14001,
            AppError::TooManyRequests => 14002,
            AppError::PayloadTooLarge => 14003,
            AppError::ServiceFailure(_) => 15000,
            AppError::ResponseFailure(_) => 15001,
            AppError::DataFailure(_) => 15002,
            AppError::ResourceFailure(_) => 15003,
        }
    }

    fn client_message(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "リクエストが正しくありません",
            AppError::InvalidRequestFormat(_) => "リクエストの形式が正しくありません",
            AppError::Unauthorized => "認証に失敗しました",
            AppError::Forbidden => "この操作は許可されていません",
            AppError::NotFound => "対象データが見つかりません",
            AppError::Conflict => "リクエストを処理できませんでした",
            AppError::PreconditionFailed(_) => "データの更新が必要です",
            AppError::Maintenance => "現在メンテナンス中です",
            AppError::UnsupportedMediaType => "対応していないリクエスト形式です",
            AppError::TooManyRequests => "しばらく時間をおいてから再度お試しください",
            AppError::PayloadTooLarge => "リクエストが大きすぎます",
            AppError::ServiceFailure(_)
            | AppError::ResponseFailure(_)
            | AppError::DataFailure(_)
            | AppError::ResourceFailure(_) => "サーバーエラーが発生しました",
        }
    }

    fn http_status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) | AppError::InvalidRequestFormat(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::PreconditionFailed(_) => StatusCode::PRECONDITION_FAILED,
            AppError::Maintenance => StatusCode::SERVICE_UNAVAILABLE,
            AppError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            AppError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::ServiceFailure(_)
            | AppError::ResponseFailure(_)
            | AppError::DataFailure(_)
            | AppError::ResourceFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log_internal(&self) {
        match self {
            AppError::BadRequest(detail) => eprintln!("[BadRequest] {}", detail),
            AppError::InvalidRequestFormat(detail) => {
                eprintln!("[InvalidRequestFormat] {}", detail)
            }
            AppError::ResponseFailure(detail) => eprintln!("[ResponseFailure] {}", detail),
            AppError::DataFailure(detail) => eprintln!("[DataFailure] {}", detail),
            AppError::ResourceFailure(detail) => eprintln!("[ResourceFailure] {}", detail),
            AppError::ServiceFailure(detail) => eprintln!("[ServiceFailure] {}", detail),
            AppError::Unauthorized => eprintln!("[Unauthorized]"),
            AppError::Forbidden => eprintln!("[Forbidden]"),
            AppError::NotFound => eprintln!("[NotFound]"),
            AppError::Conflict => eprintln!("[Conflict]"),
            AppError::PreconditionFailed(_) => eprintln!("[PreconditionFailed]"),
            AppError::Maintenance => eprintln!("[Maintenance]"),
            AppError::UnsupportedMediaType => eprintln!("[UnsupportedMediaType]"),
            AppError::TooManyRequests => eprintln!("[TooManyRequests]"),
            AppError::PayloadTooLarge => eprintln!("[PayloadTooLarge]"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let code = self.app_code();
        let error_message = self.client_message();

        self.log_internal();

        let master_data = match self {
            AppError::PreconditionFailed(master) => *master,
            _ => None,
        };

        let body = ErrorResponse {
            header: ResponseHeader {
                code,
                master: master_data,
            },
            error: error_message,
        };

        (status, Msgpack(body)).into_response()
    }
}
