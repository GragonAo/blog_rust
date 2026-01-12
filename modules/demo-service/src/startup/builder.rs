use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use crate::config::application::AppConfig;

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

    // 4. 初始化业务服务
    // let user_service = Arc::new(UserServiceImpl {
    //     redis_client: redis_client.clone(),
    //     db_pool: db_pool.clone(),
    //     id_generator: id_generator.clone(),
    // }) as Arc<dyn UserService>;

    Ok(AppState {
        redis_client,
        db_pool,
        id_generator,
        app_config: Arc::new(app_config),
    })
}
