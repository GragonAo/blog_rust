use common_core::AppError;
use reqwest::Client;
use std::sync::Arc;

use crate::{
    config::application::AppConfig, middleware::circuit_breaker::CircuitBreakerManager,
};

use super::AppState;

/// 加载应用配置
pub fn init_app_config() -> Result<AppConfig, AppError> {
    AppConfig::from_default_yaml()
}

/// 初始化应用状态
pub async fn init_app_state(app_config: AppConfig) -> Result<AppState, AppError> {
    // HTTP 客户端
    let http_client = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .build()
        .map_err(|e| AppError::internal(format!("Failed to create HTTP client: {}", e)))?;

    // 熔断器管理器
    let circuit_breaker = CircuitBreakerManager::new(
        app_config.circuit_breaker.failure_threshold,
        app_config.circuit_breaker.timeout_seconds,
    );

    Ok(AppState {
        http_client,
        circuit_breaker,
        app_config: Arc::new(app_config),
    })
}
