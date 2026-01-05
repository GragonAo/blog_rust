use reqwest::Client;
use std::sync::Arc;

use crate::{config::application::AppConfig, middleware::circuit_breaker::CircuitBreakerManager};

/// 网关应用状态
#[derive(Clone)]
pub struct AppState {
    /// HTTP 客户端（用于转发请求）
    pub http_client: Client,

    /// 熔断器管理器
    pub circuit_breaker: CircuitBreakerManager,

    /// 配置
    pub app_config: Arc<AppConfig>,
}
