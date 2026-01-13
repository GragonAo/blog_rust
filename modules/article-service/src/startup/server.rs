use axum::{Router, routing::get};
use tokio::task::JoinHandle;

use crate::routes::{article_route, authorship_route};

use super::AppState;

/// 启动 HTTP 服务器
pub fn start_http_server(app_state: AppState, bind_addr: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .merge(article_route::router())
            .nest("/authorship", authorship_route::router())
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .expect("Failed to bind HTTP server");

        println!(
            "HTTP server listening on {}",
            listener.local_addr().unwrap()
        );

        axum::serve(listener, app).await.expect("HTTP server error");
    })
}
