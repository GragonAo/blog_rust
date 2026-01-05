mod config;
mod middleware;
mod proxy;
mod startup;

use common_core::AppError;
use startup::{init_app_config, init_app_state, start_http_server};

pub use startup::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 初始化日志（输出到文件和控制台）
    let file_appender = tracing_appender::rolling::daily("logs", "gateway-service.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    use tracing_subscriber::fmt::writer::MakeWriterExt;
    let stdout = std::io::stdout.and(non_blocking);
    
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(stdout)
        .compact()
        .init();

    // 1. 加载配置
    let app_config = init_app_config()?;
    let bind_addr = app_config.server.bind_addr.clone();

    // 2. 初始化应用状态
    let app_state = init_app_state(app_config).await?;

    // 3. 启动网关服务器
    tracing::info!("🚀 Gateway starting on {}", bind_addr);
    start_http_server(app_state, bind_addr).await
}
