use common_core::AppError;
use common_tracing::application::Logs;
use common_web::application::Server;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: Server,
    pub cors: CorsConfig,
    pub jwt: JwtConfig,
    pub rate_limit: RateLimitConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub services: HashMap<String, ServiceConfig>,
    pub logs: Logs,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    #[serde(default)]
    pub whitelist_paths: Vec<String>,
}

impl JwtConfig {
    /// 检查路径是否在白名单中
    pub fn is_whitelisted(&self, path: &str) -> bool {
        self.whitelist_paths.iter().any(|whitelist| {
            // 支持精确匹配和前缀匹配
            path == whitelist || path.starts_with(&format!("{}/", whitelist))
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub path_prefix: String,
    pub timeout_seconds: u64,
}

impl AppConfig {
    pub fn from_default_yaml() -> Result<Self, AppError> {
        let config_path = "gateway-service/application.yaml";
        let content = fs::read_to_string(config_path)
            .map_err(|e| AppError::internal(format!("Failed to read config: {}", e)))?;

        let config: AppConfig = serde_yml::from_str(&content)
            .map_err(|e| AppError::internal(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }
}
