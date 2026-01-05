use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// 请求追踪中间件（增强版）
pub async fn request_tracing(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();
    let start = Instant::now();

    tracing::info!(
        method = %method,
        path = %path,
        query = %query,
        "→ Incoming request"
    );

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();
    let duration_ms = duration.as_millis();

    // 根据状态码和耗时选择日志级别
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration_ms,
            "← Response completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration_ms,
            "← Response completed with client error"
        );
    } else if duration_ms > 1000 {
        tracing::warn!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration_ms,
            "← Slow request detected"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration_ms,
            "← Response completed"
        );
    }

    response
}
