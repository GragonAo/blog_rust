use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use common_web::{domain::r::R, error::ApiError};

use crate::{domain::response::authorship::AuthorshipRes, startup::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/detail/{uid}", get(get_authorship_detail))
}

/// 获取作者信息
async fn get_authorship_detail(
    State(mut state): State<AppState>,
    Path(uid): Path<i64>,
) -> Result<Json<R<AuthorshipRes>>, ApiError> {
    let authorship = state.authorship_service.get_authorship(uid).await?;

    // 调用 gRPC 获取用户信息
    let (username, avatar_url) = state
        .user_grpc_client
        .get_user_info(authorship.uid)
        .await
        .unwrap_or_else(|_| ("Unknown".to_string(), String::new()));

    Ok(Json(R::ok(AuthorshipRes {
        id: authorship.uid.to_string(),
        name: username,
        avatar_url,
        like_count: authorship.like_count,
        fellow_count: authorship.fellow_count,
        collect_count: authorship.collect_count,
        article_count: authorship.article_count_public,
    })))
}
