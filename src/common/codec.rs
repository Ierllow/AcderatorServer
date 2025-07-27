use super::{
    config::{MSGPACK_CONTENT_TYPE, REQUEST_BODY_LIMIT_BYTES},
    error::AppError,
};
use axum::{
    async_trait,
    body::to_bytes,
    extract::{FromRequest, Request},
    http::{header::CONTENT_LENGTH, header::CONTENT_TYPE, HeaderValue},
    response::{IntoResponse, Response},
};
use serde::{de::DeserializeOwned, Serialize};

pub struct Msgpack<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Msgpack<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let content_type = parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::trim);

        if !content_type.is_some_and(|value| value.eq_ignore_ascii_case(MSGPACK_CONTENT_TYPE)) {
            return Err(AppError::UnsupportedMediaType);
        }

        if parts
            .headers
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|content_length| content_length > REQUEST_BODY_LIMIT_BYTES)
        {
            return Err(AppError::PayloadTooLarge);
        }

        let bytes = to_bytes(body, REQUEST_BODY_LIMIT_BYTES)
            .await
            .map_err(|_| AppError::PayloadTooLarge)?;
        let value = rmp_serde::from_slice(&bytes)?;

        Ok(Msgpack(value))
    }
}

impl<T> IntoResponse for Msgpack<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match rmp_serde::to_vec_named(&self.0) {
            Ok(body) => (
                [(CONTENT_TYPE, HeaderValue::from_static(MSGPACK_CONTENT_TYPE))],
                body,
            )
                .into_response(),
            Err(err) => AppError::from(err).into_response(),
        }
    }
}
