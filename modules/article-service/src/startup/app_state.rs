use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{config::application::AppConfig, services::article_service::ArticleService};

/// 应用状态
///
/// 包含所有业务服务、基础设施组件和配置
#[derive(Clone)]
pub struct AppState {
    // 业务服务
    pub article_service: Arc<dyn ArticleService>,

    // 基础设施组件
    #[allow(dead_code)]
    pub redis_client: RedisClient,
    #[allow(dead_code)]
    pub db_pool: PgPool,
    #[allow(dead_code)]
    pub id_generator: Arc<RwLock<SnowflakeIdGenerator>>,

    // 配置
    #[allow(dead_code)]
    pub app_config: Arc<AppConfig>,
}
