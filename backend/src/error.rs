use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("not found")]
    NotFound,

    #[error("db error")]
    Db(#[from] sqlx::Error),

    #[error("internal")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".into()),
            AppError::Db(e) => {
                tracing::error!("db error: {e:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal".into())
            }
            AppError::Internal(e) => {
                tracing::error!("internal: {e:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal".into())
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
