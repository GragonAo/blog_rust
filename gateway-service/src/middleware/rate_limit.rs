use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};

/// 限流器
pub type AppRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// 创建限流器
pub fn create_rate_limiter(requests_per_second: u32, burst_size: u32) -> AppRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap())
        .allow_burst(NonZeroU32::new(burst_size).unwrap());

    Arc::new(RateLimiter::direct(quota))
}

/// 限流中间件
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    limiter: axum::extract::Extension<AppRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 检查是否超过限流
    if limiter.check().is_err() {
        tracing::warn!("Rate limit exceeded for IP: {}", addr.ip());
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests, please try again later",
        )
            .into_response());
    }

    Ok(next.run(request).await)
}
