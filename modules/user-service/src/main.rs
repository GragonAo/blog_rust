pub mod config;
mod domain;
mod error;
mod grpc;
mod repository;
mod routes;
mod services;
mod startup;

pub use startup::AppState;

use common_core::AppError;
use startup::{init_app_config, init_app_state, start_grpc_server, start_http_server};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 初始化日志（输出到文件和控制台）
    let file_appender = tracing_appender::rolling::daily("logs", "user-service.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    use tracing_subscriber::fmt::writer::MakeWriterExt;
    let stdout = std::io::stdout.and(non_blocking);

    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(stdout)
        .compact()
        .init();

    tracing::info!("🚀 User Service starting...");

    // 1. 加载配置
    let app_config = init_app_config()?;
    let http_bind_addr = app_config.server.bind_addr.clone();
    let grpc_bind_addr = app_config.server.grpc_addr.clone();

    // 2. 初始化应用（基础设施 + 业务服务）
    let app_state = init_app_state(app_config).await?;

    // 3. 启动服务器
    let http_server = start_http_server(app_state.clone(), http_bind_addr);
    let grpc_server = start_grpc_server(app_state, grpc_bind_addr);

    // 4. 等待服务器运行
    let _ = tokio::try_join!(http_server, grpc_server)
        .map_err(|e| AppError::internal(format!("Server error: {}", e)))?;

    Ok(())
}
