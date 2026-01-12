use common_core::{AppError, application::Snowflake};
use common_redis::application::Redis;
use common_tracing::application::Logs;
use common_web::application::Server;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Database {
    pub fn connection_url(&self) -> String {
        if !self.username.is_empty() && !self.password.is_empty() {
            if let Some(at_pos) = self.url.rfind('@') {
                let host_part = &self.url[at_pos..];
                format!(
                    "postgres://{}:{}{}",
                    self.username, self.password, host_part
                )
            } else {
                self.url.replace(
                    "postgres://",
                    &format!("postgres://{}:{}@", self.username, self.password),
                )
            }
        } else {
            self.url.clone()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub redis: Redis,
    pub database: Database,
    pub snowflake: Snowflake,
    pub server: Server,
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
        // 尝试多个可能的路径
        let possible_paths = [
            "src/application.yaml",
            "application.yaml",
            "modules/user-service/application.yaml",
        ];
        for path in &possible_paths {
            if let Ok(config) = Self::from_yaml(path) {
                return Ok(config);
            }
        }
        Err(AppError::internal(
            "Failed to load config from any default path. Tried: src/application.yaml, modules/user-service/src/application.yaml, application.yaml",
        ))
    }
}
