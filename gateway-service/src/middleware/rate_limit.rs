use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, keyed::DefaultKeyedStateStore},
};
use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};

/// 限流器（按 IP 限流）
pub type AppRateLimiter = Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>;

/// 创建限流器（按 IP）
pub fn create_rate_limiter(requests_per_second: u32, burst_size: u32) -> AppRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap())
        .allow_burst(NonZeroU32::new(burst_size).unwrap());

    Arc::new(RateLimiter::keyed(quota))
}

/// 限流中间件（按 IP 限流）
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    limiter: axum::extract::Extension<AppRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = addr.ip();

    // 按 IP 检查是否超过限流
    if limiter.check_key(&ip).is_err() {
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests, please try again later",
        )
            .into_response());
    }

    Ok(next.run(request).await)
}
