pub mod config;
mod domain;
mod grpc;
mod repository;
mod routes;
mod services;
mod startup;

use common_core::AppError;
use common_tracing::TracingService;
use startup::{init_app_config, init_app_state, start_http_server};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. 加载配置
    let app_config = init_app_config()?;
    let service_name = app_config.server.name.clone();
    // 初始化日志
    let _guard = TracingService::init(&app_config.logs);
    tracing::info!("🚀 {} Service starting...", service_name);

    let http_bind_addr = app_config.server.bind_addr.clone();

    // 2. 初始化应用（基础设施 + 业务服务）
    let app_state = match init_app_state(app_config).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("{} service startup failed: {}", service_name, e);
            return Err(e);
        }
    };

    // 3. 启动服务器
    let http_server = start_http_server(app_state.clone(), http_bind_addr);

    // 4. 等待服务器运行
    let _ = tokio::try_join!(http_server)
        .map_err(|e| AppError::internal(format!("Server error: {}", e)))?;

    Ok(())
}
