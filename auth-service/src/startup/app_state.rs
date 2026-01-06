use common_proto::user::user_service_client::UserServiceClient;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use std::sync::{Arc, Mutex};
use tonic::transport::Channel;

use crate::{config::application::AppConfig, services::login_service::LoginService};

/// 应用状态
///
/// 包含所有业务服务、基础设施组件、gRPC 客户端和配置
/// 通过 Clone（内部 Arc）可以高效共享
#[derive(Clone)]
pub struct AppState {
    // 业务服务
    pub login_service: Arc<dyn LoginService>,

    // 基础设施组件
    pub redis_client: RedisClient,
    pub id_generator: Arc<Mutex<SnowflakeIdGenerator>>,

    // gRPC 客户端 (tonic 客户端本身可克隆，无需 Arc)
    pub user_grpc_client: UserServiceClient<Channel>,

    // 配置
    pub app_config: Arc<AppConfig>,
}
