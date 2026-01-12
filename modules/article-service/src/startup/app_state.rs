use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::application::AppConfig;

/// 应用状态
///
/// 包含所有业务服务、基础设施组件和配置
#[derive(Clone)]
pub struct AppState {
    // 业务服务
    // pub user_service: Arc<dyn UserService>,

    // 基础设施组件
    pub redis_client: RedisClient,
    pub db_pool: PgPool,
    pub id_generator: Arc<RwLock<SnowflakeIdGenerator>>,

    // 配置
    pub app_config: Arc<AppConfig>,
}
