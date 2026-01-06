use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use std::sync::{Arc, Mutex};

use crate::{
    config::application::AppConfig,
    grpc::user_client::UserServiceGrpcClient,
    services::login_service::{LoginService, LoginServiceImpl},
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

    // 2. 初始化 UserService gRPC 客户端 (tonic 客户端本身可克隆)
    let user_grpc_client =
        UserServiceGrpcClient::new(app_config.services.user_service_grpc.clone()).await?;

    // 3. 初始化 ID 生成器
    let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(
        app_config.snowflake.machine_id,
        app_config.snowflake.node_id,
    )));

    // 4. 初始化业务服务
    let login_service = Arc::new(LoginServiceImpl {
        redis_client: redis_client.clone(),
        id_generator: id_generator.clone(),
    }) as Arc<dyn LoginService>;

    Ok(AppState {
        login_service,
        redis_client,
        id_generator,
        user_grpc_client,
        app_config: Arc::new(app_config),
    })
}
