use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use crate::{
    config::application::AppConfig,
    grpc::user_client::UserServiceGrpcClient,
    services::{
        article_service::{ArticleService, ArticleServiceImpl},
        authorship_service::{AuthorshipService, AuthorshipServiceImpl},
    },
};

use super::AppState;

/// 加载应用配置
pub fn init_app_config() -> Result<AppConfig, AppError> {
    AppConfig::from_default_yaml()
}

/// 初始化应用（基础设施 + 业务服务）
pub async fn init_app_state(app_config: AppConfig) -> Result<AppState, AppError> {
    // 1. 初始化 Redis 客户端
    let redis_client = RedisClient::new(app_config.redis.clone()).await?;

    // 2. 初始化数据库连接池
    let db_url = app_config.database.connection_url();
    let db_pool = PgPoolOptions::new()
        .max_connections(app_config.database.max_connections)
        .min_connections(app_config.database.min_connections)
        .connect(&db_url)
        .await
        .map_err(|e| AppError::Db(format!("Failed to connect to database: {}", e)))?;

    // 3. 初始化 ID 生成器
    let id_generator = Arc::new(tokio::sync::RwLock::new(SnowflakeIdGenerator::new(
        app_config.snowflake.machine_id,
        app_config.snowflake.node_id,
    )));

    // 4. 初始化 gRPC 客户端
    let user_grpc_client =
        UserServiceGrpcClient::new(app_config.services.user_service_grpc.clone()).await?;

    // 5. 先初始化 AuthorshipService
    let authorship_service = Arc::new(AuthorshipServiceImpl {
        redis_client: redis_client.clone(),
        db_pool: db_pool.clone(),
        id_generator: id_generator.clone(),
    }) as Arc<dyn AuthorshipService>;

    // 6. 再初始化 ArticleService（注入 AuthorshipService）
    let article_service = Arc::new(ArticleServiceImpl {
        redis_client: redis_client.clone(),
        db_pool: db_pool.clone(),
        id_generator: id_generator.clone(),
    }) as Arc<dyn ArticleService>;

    Ok(AppState {
        article_service,
        authorship_service,
        user_grpc_client,
        redis_client,
        db_pool,
        id_generator,
        app_config: Arc::new(app_config),
    })
}
