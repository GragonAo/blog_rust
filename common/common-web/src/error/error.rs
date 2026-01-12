use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use common_core::AppError;

use crate::domain::r::R;

pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log the error
        tracing::error!(error = ?self.0, "API Error occurred");

        let (status, msg) = match &self.0 {
            AppError::Internal(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.clone()),
            AppError::Redis(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.clone()),
            AppError::Db(s) => (StatusCode::INTERNAL_SERVER_ERROR, s.clone()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(R::<()>::error(msg, status.as_u16()))).into_response()
    }
}
