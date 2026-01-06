use axum::{Json, Router, extract::State, routing::get};
use common_web::domain::r::R;

use crate::error::ApiError;

use crate::{domain::response::user::UserInfoResponse, startup::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/user", get(get_user_info))
}

async fn get_user_info(
    State(app_state): State<AppState>,
) -> Result<Json<R<UserInfoResponse>>, ApiError> {
    let user_info_opt = app_state.user_service.get_user_info(1).await?;
    if let Some(ui) = user_info_opt {
        Ok(Json(R::ok(UserInfoResponse::from(ui))))
    } else {
        Err(ApiError(common_core::AppError::internal("User not found")))
    }
}
