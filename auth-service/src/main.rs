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
    // 1. 加载配置
    let app_config = init_app_config()?;
    let bind_addr = app_config.server.bind_addr.clone();

    // 2. 初始化应用（基础设施 + 业务服务）
    let app_state = init_app_state(app_config).await?;

    // 3. 启动 HTTP 服务器
    start_http_server(app_state, bind_addr).await
}
