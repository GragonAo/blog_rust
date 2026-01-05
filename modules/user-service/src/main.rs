use std::sync::{Arc, Mutex};
pub mod config;
mod domain;
mod error;
mod repository;
mod routes;
mod services;
use axum::{Router, routing::get};
use common_core::AppError;
use snowflake::SnowflakeIdGenerator;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{
    config::application::AppConfig,
    services::user_service::{UserService, UserServiceImpl},
};
use common_redis::RedisClient;
use routes::user_router;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,

    pub app_config: Arc<AppConfig>,
}

#[derive(Clone)]
struct AppInfrastructureState {
    redis_client: RedisClient,
    db_pool: PgPool,
    id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
}

fn init_app_config() -> Result<AppConfig, AppError> {
    // 从 YAML 文件加载配置
    AppConfig::from_default_yaml()
}

async fn init_infrastructure(app_config: &AppConfig) -> Result<AppInfrastructureState, AppError> {
    // 初始化 Redis 客户端
    let redis_client = RedisClient::new(app_config.redis.clone()).await?;

    // 初始化数据库连接池
    let db_url = app_config.database.connection_url();
    let db_pool = PgPoolOptions::new()
        .max_connections(app_config.database.max_connections)
        .min_connections(app_config.database.min_connections)
        .connect(&db_url)
        .await
        .map_err(|e| AppError::Db(format!("Failed to connect to database: {}", e)))?;

    // 初始化 ID 生成器
    let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(
        app_config.snowflake.machine_id,
        app_config.snowflake.node_id,
    )));

    Ok(AppInfrastructureState {
        redis_client,
        db_pool,
        id_generator,
    })
}

/// 初始化系统服务和上下文
fn init_service(app_infra: &AppInfrastructureState, app_config: AppConfig) -> AppState {
    let user_service = Arc::new(UserServiceImpl {
        redis_client: app_infra.redis_client.clone(),
        db_pool: app_infra.db_pool.clone(),
        id_generator: app_infra.id_generator.clone(),
    }) as Arc<dyn UserService>;
    AppState {
        user_service,
        app_config: Arc::new(app_config),
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_config = init_app_config()?;
    let serivce_bind_addr = app_config.server.bind_addr.clone();

    let app_infra = init_infrastructure(&app_config).await?;
    let app_state = init_service(&app_infra, app_config);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(user_router::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(serivce_bind_addr).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
