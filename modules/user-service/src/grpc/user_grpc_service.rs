use common_proto::user::{
    GetUserInfoRequest, GetUserInfoResponse,
    user_service_server::{UserService as UserServiceTrait, UserServiceServer},
};
use tonic::{Request, Response, Status};

use crate::startup::AppState;

/// gRPC 服务实现
///
/// 持有完整的 AppState，可以访问所有业务服务和配置
/// 这样当添加新的 gRPC 方法时，可以灵活调用多个业务服务
pub struct UserGrpcService {
    app_state: AppState,
}

impl UserGrpcService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub fn into_server(self) -> UserServiceServer<Self> {
        UserServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl UserServiceTrait for UserGrpcService {
    async fn get_user_info(
        &self,
        request: Request<GetUserInfoRequest>,
    ) -> Result<Response<GetUserInfoResponse>, Status> {
        let req = request.into_inner();

        // 通过 app_state 访问业务服务
        // 未来如果需要调用多个服务，可以这样：
        // - self.app_state.user_service
        // - self.app_state.order_service (未来添加)
        // - self.app_state.app_config (访问配置)
        let user = self
            .app_state
            .user_service
            .get_user_info(req.user_id as u64)
            .await
            .map_err(|e| Status::internal(format!("Failed to get user info: {}", e)))?;

        match user {
            Some(user) => {
                let response = GetUserInfoResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.to_rfc3339(),
                    updated_at: user.updated_at.to_rfc3339(),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }
}
