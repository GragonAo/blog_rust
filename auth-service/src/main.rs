use std::sync::{Arc, Mutex};
mod application;
mod domain;
mod error;
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
pub struct AppState {
    pub login_service: Arc<dyn LoginService>,

    pub app_config: Arc<AppConfig>,
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

    let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(
        app_config.snowflake.machine_id,
        app_config.snowflake.node_id,
    )));

    Ok(AppInfrastructureState {
        redis_client,
        id_generator,
    })
}

/// 初始化系统服务和上下文
fn init_service(app_infra: &AppInfrastructureState, app_config: AppConfig) -> AppState {
    let login_service = Arc::new(LoginServiceImpl {
        redis_client: app_infra.redis_client.clone(),
        db_pool: "postgres://todo".into(),
        id_generator: app_infra.id_generator.clone(),
    }) as Arc<dyn LoginService>;

    AppState {
        login_service,
        app_config: Arc::new(app_config),
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_config = init_app_config();
    let serivce_bind_addr = app_config.bind_addr.clone();

    let app_infra = init_infrastructure(&app_config).await?;
    let app_state = init_service(&app_infra, app_config);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(login_router::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(serivce_bind_addr).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
