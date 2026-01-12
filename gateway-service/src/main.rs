mod config;
mod middleware;
mod proxy;
mod startup;

use common_core::AppError;
use common_tracing::TracingService;
use startup::{init_app_config, init_app_state, start_http_server};

pub use startup::AppState;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1. 加载配置
    let app_config = init_app_config()?;

    // 初始化日志
    let _guard = TracingService::init(&app_config.logs);

    let bind_addr = app_config.server.bind_addr.clone();
    let server_name = app_config.server.name.clone();

    // 2. 初始化应用状态
    let app_state = init_app_state(app_config).await?;

    // 3. 启动网关服务器
    tracing::info!("🚀 {} starting on {}", server_name, bind_addr);
    start_http_server(app_state, bind_addr).await
}
