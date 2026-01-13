use common_core::AppError;
use common_proto::user::{user_service_client::UserServiceClient, UserInfoReq};
use std::time::Duration;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct UserServiceGrpcClient {
    client: UserServiceClient<Channel>,
}

impl UserServiceGrpcClient {
    /// 创建用户服务 gRPC 客户端
    pub async fn new(addr: String) -> Result<Self, AppError> {
        let channel = Channel::from_shared(addr)
            .map_err(|e| AppError::internal(format!("Invalid gRPC address: {}", e)))?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .connect_lazy();

        let client = UserServiceClient::new(channel);
        Ok(Self { client })
    }

    /// 获取用户信息
    pub async fn get_user_info(&mut self, user_id: i64) -> Result<(String, String), AppError> {
        let request = tonic::Request::new(UserInfoReq { user_id });
        
        let response = self
            .client
            .get_user_info(request)
            .await
            .map_err(|e| AppError::internal(format!("Failed to call user service: {}", e)))?;

        let user_info = response.into_inner();
        
        // 返回 (username, avatar_url) 组合
        // 注意：avatar_url 在当前 proto 中可能没有，这里先用 username 替代
        Ok((
            user_info.username.clone(),
            format!("https://avatar.example.com/{}", user_info.username),
        ))
    }
}
