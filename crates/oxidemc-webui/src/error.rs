use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// One error type for every handler. Maps core errors → HTTP status codes
/// and always returns a JSON body the frontend can render inline (no toasts).
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("server not found: {0}")]
    NotFound(String),

    #[error("server is already running")]
    AlreadyRunning,

    #[error("server is not running")]
    NotRunning,

    #[error("RCON is disabled for this server")]
    RconDisabled,

    #[error("validation: {0}")]
    Validation(String),

    #[error(transparent)]
    Config(#[from] oxidemc_core::config::ConfigError),

    #[error(transparent)]
    Download(#[from] oxidemc_core::downloader::DownloadError),

    #[error(transparent)]
    Server(#[from] oxidemc_core::server::ServerError),

    #[error(transparent)]
    Rcon(#[from] oxidemc_core::rcon::RconError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::AlreadyRunning | ApiError::NotRunning | ApiError::RconDisabled => {
                StatusCode::CONFLICT
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
