use std::sync::{Arc, Mutex};

mod application;
mod domain;
mod routes;
mod services;

use axum::{Router, routing::get};
use common_core::AppError;
use snowflake::SnowflakeIdGenerator;

use crate::{
    application::AppConfig,
    services::login_service::{LoginService, LoginServiceImpl},
};
use common_redis::RedisClient;
use routes::login_router;

#[derive(Clone)]
pub struct AppServiceState {
    pub login_service: Arc<dyn LoginService>,
}

#[derive(Clone)]
struct AppInfrastructureState {
    redis_client: Arc<RedisClient>,
    id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
}

// 初始化配置文件（目前使用默认值，后续可改为从 YAML/环境加载）
fn init_app_config() -> AppConfig {
    AppConfig::default()
}

async fn init_infrastructure(app_config: &AppConfig) -> Result<AppInfrastructureState, AppError> {
    let redis_client = RedisClient::new(app_config.redis.clone())
        .await
        .map(Arc::new)?;

    // 从配置中读取 machine_id 和 node_id (假设配置中有这些字段)
    let machine_id = 1; // app_config.snowflake.machine_id
    let node_id = 1;

    let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(machine_id, node_id)));

    Ok(AppInfrastructureState {
        redis_client,
        id_generator,
    })
}

/// 初始化系统服务
fn init_service(app_infra: &AppInfrastructureState) -> AppServiceState {
    let login_service = Arc::new(LoginServiceImpl {
        redis_client: app_infra.redis_client.clone(),
        db_pool: "postgres://todo".into(),
        id_generator: app_infra.id_generator.clone(),
    }) as Arc<dyn LoginService>;

    AppServiceState { login_service }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_config = init_app_config();
    let app_infra = init_infrastructure(&app_config).await?;
    let app_state = init_service(&app_infra);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(login_router::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&app_config.bind_addr).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
