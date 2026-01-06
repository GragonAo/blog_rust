use axum::{Router, routing::get};
use common_core::AppError;

use crate::routes::login_router;

use super::AppState;

/// 启动 HTTP 服务器
pub async fn start_http_server(app_state: AppState, bind_addr: String) -> Result<(), AppError> {
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .merge(login_router::router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}
