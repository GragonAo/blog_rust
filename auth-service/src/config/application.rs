use common_core::{AppError, application::Snowflake};
use common_redis::application::Redis;
use common_tracing::application::Logs;
use common_web::application::Server;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct JWT {
    pub secret: String,
    pub expiration_hours: u64,
    pub refresh_expiration_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Services {
    pub user_service_grpc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub redis: Redis,
    pub snowflake: Snowflake,
    pub jwt: JWT,
    pub server: Server,
    pub services: Services,
    pub logs: Logs,
}

impl AppConfig {
    /// 从 YAML 文件加载配置
    pub fn from_yaml(path: &str) -> Result<Self, AppError> {
        let content = fs::read_to_string(path).map_err(|e| {
            AppError::internal(format!("Failed to read config file {}: {}", path, e))
        })?;

        let config: AppConfig = serde_yml::from_str(&content)
            .map_err(|e| AppError::internal(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// 从默认路径加载配置
    pub fn from_default_yaml() -> Result<Self, AppError> {
        let possible_paths = [
            "src/application.yaml",
            "application.yaml",
            "auth-service/application.yaml",
        ];
        for path in &possible_paths {
            if let Ok(config) = Self::from_yaml(path) {
                return Ok(config);
            }
        }
        Err(AppError::internal(
            "Failed to load config from any default path. Tried: src/application.yaml, application.yaml",
        ))
    }
}
