use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use common_core::utils::jwt_utils::JwtUtils;

use crate::AppState;

/// JWT 验证中间件（基于白名单）
pub async fn jwt_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // 检查是否在白名单中
    if state.app_config.jwt.is_whitelisted(path) {
        tracing::debug!("Path {} is whitelisted, skipping JWT verification", path);
        return Ok(next.run(request).await);
    }

    // 从 Authorization header 获取 token
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("Missing or invalid Authorization header for path: {}", path);
            (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
                .into_response()
        })?;

    // 验证 JWT
    let _claims = JwtUtils::verify_token(state.app_config.jwt.secret.clone(), token.to_string())
        .map_err(|_| {
            tracing::warn!("Invalid or expired token for path: {}", path);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
        })?;

    // 验证通过，继续处理请求
    tracing::debug!("JWT verification passed for path: {}", path);
    Ok(next.run(request).await)
}
