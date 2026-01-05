use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use common_web::domain::r::R;

use crate::{error::ApiError, startup::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/test/user/{id}", get(test_get_user))
}

/// 测试调用 user-service 获取用户信息
async fn test_get_user(
    State(app_state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<R<String>>, ApiError> {
    let mut client = app_state.user_grpc_client.clone();

    // 调用 user-service
    let user_info = client
        .get_user_info(user_id)
        .await
        .map_err(|e| ApiError(e))?;

    match user_info {
        Some(user) => {
            let msg = format!(
                "User found via gRPC: id={}, username={}, email={:?}",
                user.id, user.username, user.email
            );
            Ok(Json(R::ok(msg)))
        }
        None => Ok(Json(R::ok(format!("User {} not found", user_id)))),
    }
}
