use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use std::time::Duration;

use crate::{AppState, config::application::ServiceConfig};

/// 代理转发请求到后端服务
pub async fn proxy_request(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read body: {}", e),
            )
                .into_response()
        })?;

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

    // 构建目标 URL（去掉路径前缀），保留查询字符串
    let target_path = path.strip_prefix(&service.path_prefix).unwrap_or(path);
    let target_url = if let Some(query) = uri.query() {
        format!("{}{}?{}", service.url, target_path, query)
    } else {
        format!("{}{}", service.url, target_path)
    };

    // 使用熔断器保护调用
    let http_client = state.http_client.clone();
    let headers_clone = headers.clone();
    let target_url_clone = target_url.clone();
    let service_clone = service.clone();
    let body_clone = body.clone();

    let result = state
        .circuit_breaker
        .call(&service.path_prefix, || async move {
            forward_request(
                &http_client,
                &method,
                &target_url_clone,
                &headers_clone,
                &body_clone,
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
    method: &Method,
    target_url: &str,
    headers: &HeaderMap,
    body: &[u8],
    service: &ServiceConfig,
) -> Result<Response, reqwest::Error> {
    // 根据 HTTP 方法构建请求
    let mut request_builder = match *method {
        Method::GET => client.get(target_url),
        Method::POST => client.post(target_url),
        Method::PUT => client.put(target_url),
        Method::DELETE => client.delete(target_url),
        Method::PATCH => client.patch(target_url),
        Method::HEAD => client.head(target_url),
        Method::OPTIONS => client.request(Method::OPTIONS, target_url),
        _ => client.request(method.clone(), target_url),
    };

    request_builder = request_builder.timeout(Duration::from_secs(service.timeout_seconds));

    // 转发必要的请求头
    for (key, value) in headers.iter() {
        if should_forward_header(key.as_str()) {
            request_builder = request_builder.header(key, value);
        }
    }

    // 添加请求体（如果有）
    if !body.is_empty() {
        request_builder = request_builder.body(body.to_vec());
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
