use common_core::AppError;
use common_proto::user::{GetUserInfoRequest, user_service_client::UserServiceClient};
use std::time::Duration;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct UserServiceGrpcClient {
    client: UserServiceClient<Channel>,
}

impl UserServiceGrpcClient {
    /// 创建 gRPC 客户端
    ///
    /// 特性：
    /// - 自动重连：连接断开时会自动重试
    /// - Keepalive：保持连接活跃
    /// - 超时控制：避免长时间等待
    /// - 懒加载：首次使用时才真正连接
    pub async fn new(addr: String) -> Result<Self, AppError> {
        let channel = Channel::from_shared(addr)
            .map_err(|e| AppError::internal(format!("Invalid gRPC address: {}", e)))?
            .connect_timeout(Duration::from_secs(5)) // 连接超时
            .timeout(Duration::from_secs(10)) // 请求超时
            .http2_keep_alive_interval(Duration::from_secs(30)) // Keepalive 间隔
            .keep_alive_timeout(Duration::from_secs(10)) // Keepalive 超时
            .connect_lazy(); // 懒加载，首次调用时连接

        let client = UserServiceClient::new(channel);

        Ok(Self { client })
    }

    pub async fn get_user_info(&mut self, user_id: i64) -> Result<Option<UserInfo>, AppError> {
        let request = tonic::Request::new(GetUserInfoRequest { user_id });

        match self.client.get_user_info(request).await {
            Ok(response) => {
                let user = response.into_inner();
                Ok(Some(UserInfo {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                }))
            }
            Err(status) => {
                if status.code() == tonic::Code::NotFound {
                    Ok(None)
                } else {
                    Err(AppError::internal(format!(
                        "Failed to get user info: {}",
                        status.message()
                    )))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
