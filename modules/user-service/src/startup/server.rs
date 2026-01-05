use axum::{routing::get, Router};
use tokio::task::JoinHandle;

use crate::grpc::user_grpc_service::UserGrpcService;
use crate::routes::user_router;

use super::AppState;

/// 启动 HTTP 服务器
pub fn start_http_server(app_state: AppState, bind_addr: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .merge(user_router::router())
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .expect("Failed to bind HTTP server");

        println!(
            "HTTP server listening on {}",
            listener.local_addr().unwrap()
        );

        axum::serve(listener, app)
            .await
            .expect("HTTP server error");
    })
}

/// 启动 gRPC 服务器
pub fn start_grpc_server(app_state: AppState, bind_addr: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let grpc_service = UserGrpcService::new(app_state);

        let addr = bind_addr
            .parse()
            .expect("Failed to parse gRPC address");

        println!("gRPC server listening on {}", addr);

        tonic::transport::Server::builder()
            .add_service(grpc_service.into_server())
            .serve(addr)
            .await
            .expect("gRPC server error");
    })
}
