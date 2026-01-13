use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    config::application::AppConfig,
    grpc::user_client::UserServiceGrpcClient,
    services::{article_service::ArticleService, authorship_service::AuthorshipService},
};

/// 应用状态
///
/// 包含所有业务服务、基础设施组件和配置
#[derive(Clone)]
pub struct AppState {
    // 业务服务
    pub article_service: Arc<dyn ArticleService>,
    pub authorship_service: Arc<dyn AuthorshipService>,

    // gRPC 客户端
    pub user_grpc_client: UserServiceGrpcClient,

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
