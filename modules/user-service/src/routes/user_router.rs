use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};
use common_web::{domain::r::R, error::ApiError};

use crate::{domain::response::user::UserInfoResponse, startup::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/info", get(get_user_info))
}

async fn get_user_info(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<R<UserInfoResponse>>, ApiError> {
    let user_id = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| ApiError(common_core::AppError::internal("User not authenticated")))?;

    let user_info_opt = app_state.user_service.get_user_info(user_id).await?;
    if let Some(ui) = user_info_opt {
        Ok(Json(R::ok(UserInfoResponse::from(ui))))
    } else {
        Err(ApiError(common_core::AppError::internal("User not found")))
    }
}
