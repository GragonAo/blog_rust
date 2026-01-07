mod config;
mod domain;
mod error;
mod grpc;
mod routes;
mod services;
mod startup;

use common_core::AppError;
use startup::{init_app_config, init_app_state, start_http_server};

pub use startup::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 初始化日志（输出到文件和控制台）
    let file_appender = tracing_appender::rolling::daily("logs", "auth-service.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    use tracing_subscriber::fmt::writer::MakeWriterExt;
    let stdout = std::io::stdout.and(non_blocking);

    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(stdout)
        .compact()
        .init();

    tracing::info!("🚀 Auth Service starting...");

    // 1. 加载配置
    let app_config = init_app_config()?;
    let bind_addr = app_config.server.bind_addr.clone();

    // 2. 初始化应用（基础设施 + 业务服务）
    let app_state = match init_app_state(app_config).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Auth service startup failed: {}", e);
            return Err(e);
        }
    };

    // 3. 启动 HTTP 服务器
    start_http_server(app_state, bind_addr).await
}
