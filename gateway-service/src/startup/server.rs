use axum::{
    http::{HeaderName, HeaderValue, Method},
    middleware,
    routing::{any, get},
    Router,
};
use common_core::AppError;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    middleware::{
        auth::jwt_auth,
        rate_limit::{create_rate_limiter, rate_limit_middleware},
        tracing::request_tracing,
    },
    proxy::proxy_request,
};

use super::AppState;

/// 启动 HTTP 服务器
pub async fn start_http_server(app_state: AppState, bind_addr: String) -> Result<(), AppError> {
    // 创建限流器
    let rate_limiter = create_rate_limiter(
        app_state.app_config.rate_limit.requests_per_second,
        app_state.app_config.rate_limit.burst_size,
    );

    // 配置 CORS
    let cors_config = &app_state.app_config.cors;
    let mut cors_layer = CorsLayer::new();

    // 配置允许的源
    if cors_config.allowed_origins.contains(&"*".to_string()) {
        cors_layer = cors_layer.allow_origin(Any);
    } else {
        let origins: Result<Vec<HeaderValue>, _> = cors_config
            .allowed_origins
            .iter()
            .map(|origin| origin.parse())
            .collect();
        cors_layer = cors_layer.allow_origin(
            origins.map_err(|e| AppError::internal(format!("Invalid CORS origin: {}", e)))?,
        );
    }

    // 配置允许的方法
    let methods: Result<Vec<Method>, _> = cors_config
        .allowed_methods
        .iter()
        .map(|m| m.parse())
        .collect();
    cors_layer = cors_layer.allow_methods(
        methods.map_err(|e| AppError::internal(format!("Invalid HTTP method: {}", e)))?,
    );

    // 配置允许的请求头
    if cors_config.allowed_headers.contains(&"*".to_string()) {
        cors_layer = cors_layer.allow_headers(Any);
    } else {
        let headers: Result<Vec<HeaderName>, _> = cors_config
            .allowed_headers
            .iter()
            .map(|h| h.parse())
            .collect();
        cors_layer = cors_layer.allow_headers(
            headers.map_err(|e| AppError::internal(format!("Invalid CORS header: {}", e)))?,
        );
    }

    // 配置暴露的响应头
    let expose_headers: Result<Vec<HeaderName>, _> = cors_config
        .exposed_headers
        .iter()
        .map(|h| h.parse())
        .collect();
    cors_layer = cors_layer.expose_headers(
        expose_headers.map_err(|e| AppError::internal(format!("Invalid expose header: {}", e)))?,
    );

    // 配置凭证
    if cors_config.allow_credentials {
        cors_layer = cors_layer.allow_credentials(true);
    }

    // 配置预检请求缓存时间
    cors_layer = cors_layer.max_age(std::time::Duration::from_secs(cors_config.max_age));

    tracing::info!(
        "CORS configured: origins={:?}, methods={:?}, credentials={}",
        cors_config.allowed_origins,
        cors_config.allowed_methods,
        cors_config.allow_credentials
    );

    // 构建路由（统一应用中间件，通过白名单控制鉴权）
    let app = Router::new()
        .route("/health", get(|| async { "Gateway OK" }))
        // 所有 API 路由统一处理
        .route(
            "/api/{*path}",
            any(proxy_request)
                // JWT 验证（白名单路径自动跳过）
                .layer(middleware::from_fn_with_state(app_state.clone(), jwt_auth))
                // 请求追踪
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    request_tracing,
                )),
        )
        // 全局中间件（从下往上执行）
        .layer(cors_layer) // 应用 CORS 配置
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(axum::Extension(rate_limiter))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("✅ Gateway listening on {}", listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
