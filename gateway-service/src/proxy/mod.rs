use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use std::time::Duration;

use crate::{config::application::ServiceConfig, AppState};

/// 代理转发请求到后端服务
pub async fn proxy_request(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    _body: String,
) -> Result<Response, Response> {
    // 根据路径前缀确定目标服务
    let path = uri.path();
    let service = state
        .app_config
        .services
        .values()
        .find(|s| path.starts_with(&s.path_prefix))
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("No service found for path: {}", path),
            )
                .into_response()
        })?;

    // 构建目标 URL（去掉路径前缀）
    let target_path = path.strip_prefix(&service.path_prefix).unwrap_or(path);
    let target_url = format!("{}{}", service.url, target_path);

    // 使用熔断器保护调用
    let http_client = state.http_client.clone();
    let headers_clone = headers.clone();
    let target_url_clone = target_url.clone();
    let service_clone = service.clone();

    let result = state
        .circuit_breaker
        .call(&service.path_prefix, || async move {
            forward_request(
                &http_client,
                &target_url_clone,
                &headers_clone,
                "",
                &service_clone,
            )
            .await
        })
        .await;

    match result {
        Ok(response) => Ok(response),
        Err(crate::middleware::circuit_breaker::CircuitBreakerError::Open) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Service temporarily unavailable",
        )
            .into_response()),
        Err(crate::middleware::circuit_breaker::CircuitBreakerError::ServiceError(e)) => {
            Err((StatusCode::BAD_GATEWAY, format!("Backend error: {}", e)).into_response())
        }
    }
}

/// 转发请求到后端服务
async fn forward_request(
    client: &Client,
    target_url: &str,
    headers: &HeaderMap,
    _body: &str,
    service: &ServiceConfig,
) -> Result<Response, reqwest::Error> {
    let mut request_builder = client
        .get(target_url)
        .timeout(Duration::from_secs(service.timeout_seconds));

    // 转发必要的请求头
    for (key, value) in headers.iter() {
        if should_forward_header(key.as_str()) {
            request_builder = request_builder.header(key, value);
        }
    }

    let backend_response = request_builder.send().await?;

    // 构建响应
    let status = backend_response.status();
    let mut response_builder = Response::builder().status(status);

    // 转发响应头
    for (key, value) in backend_response.headers().iter() {
        if should_forward_header(key.as_str()) {
            response_builder = response_builder.header(key, value);
        }
    }

    let body = backend_response.text().await?;
    Ok(response_builder.body(Body::from(body)).unwrap())
}

/// 判断是否应该转发该请求头
fn should_forward_header(key: &str) -> bool {
    !matches!(
        key.to_lowercase().as_str(),
        "host" | "connection" | "transfer-encoding" | "content-length"
    )
}
