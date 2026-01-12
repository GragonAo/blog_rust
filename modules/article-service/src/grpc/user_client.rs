use common_core::AppError;
use common_proto::user::user_service_client::UserServiceClient;
use std::time::Duration;
use tonic::transport::Channel;

#[derive(Clone)]
#[allow(dead_code)]
pub struct UserServiceGrpcClient {
    client: UserServiceClient<Channel>,
}

impl UserServiceGrpcClient {
    /// 创建 用户服务 gRPC 客户端
    #[allow(dead_code)]
    pub async fn new(addr: String) -> Result<UserServiceClient<Channel>, AppError> {
        let channel = Channel::from_shared(addr)
            .map_err(|e| AppError::internal(format!("Invalid gRPC address: {}", e)))?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10))
            .connect_lazy();

        let client = UserServiceClient::new(channel);
        Ok(client)
    }
}
