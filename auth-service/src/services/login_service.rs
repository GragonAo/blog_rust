use async_trait::async_trait;
use common_core::AppError;
use common_proto::user::{RegisterType, RegisterUserReq, user_service_client::UserServiceClient};
use common_redis::RedisClient;
use common_web3::{Web3Recover, chain::Chain};
use snowflake::SnowflakeIdGenerator;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Channel;

const LOGIN_WEB3_NONCE_CACHE: &str = "blog:auth:login:web3:nonce";
const NONCE_EXPIRATION_SECONDS: u64 = 300;

#[async_trait]
pub trait LoginService: Send + Sync {
    async fn get_login_web3_nonce(
        &self,
        chain_id: i64,
        address: String,
    ) -> Result<String, AppError>;
    async fn login_web3_wallet(
        &self,
        signature: String,
        message: String,
    ) -> Result<(i64, String), AppError>;

    /// Web3 用户注册或获取用户ID
    async fn register_or_get_web3_user(
        &self,
        user_grpc_client: &UserServiceClient<Channel>,
        chain_id: i64,
        address: String,
    ) -> Result<i64, AppError>;
}

pub struct LoginServiceImpl {
    pub redis_client: RedisClient,
    pub id_generator: Arc<RwLock<SnowflakeIdGenerator>>,
}

#[async_trait]
impl LoginService for LoginServiceImpl {
    async fn get_login_web3_nonce(
        &self,
        chain_id: i64,
        address: String,
    ) -> Result<String, AppError> {
        let nonce_id = self.id_generator.write().await.generate().to_string();

        let redis_key = format!("{}:{}", LOGIN_WEB3_NONCE_CACHE, nonce_id);

        // 将 chain_id 和 address 组合存储
        let value = format!("{}:{}", chain_id, address);

        self.redis_client
            .set_ex(&redis_key, &value, NONCE_EXPIRATION_SECONDS)
            .await?;

        Ok(nonce_id)
    }

    async fn login_web3_wallet(
        &self,
        signature: String,
        message: String,
    ) -> Result<(i64, String), AppError> {
        // 1. 从消息中提取 Nonce
        let nonce = message
            .split(": ")
            .last()
            .ok_or_else(|| AppError::Internal("Invalid message format".into()))?;

        let redis_key = format!("{}:{}", LOGIN_WEB3_NONCE_CACHE, nonce);

        // 2. 获取并立即删除
        let cached_data = self
            .redis_client
            .get_str(&redis_key)
            .await?
            .ok_or_else(|| AppError::Internal("Nonce expired or invalid".into()))?;

        self.redis_client.del(&redis_key).await?;

        // 3. 解析缓存的数据 "chain_id:address"
        let parts: Vec<&str> = cached_data.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AppError::Internal("Corrupted session data".into()));
        }

        let chain_id = parts[0]
            .parse::<i64>()
            .map_err(|_| AppError::Internal("Invalid chain ID".into()))?;
        let expected_address = parts[1].to_lowercase();

        // 4. Web3 恢复地址
        let chain = Chain::try_from(chain_id)?;
        let recovered_addr = Web3Recover::get_address(chain, &message, &signature)?;

        // 5. 恢复出的地址必须等于申请 Nonce 时填写的地址
        if recovered_addr.to_lowercase() != expected_address {
            return Err(AppError::Internal("Signature address mismatch".into()));
        }

        Ok((chain_id, recovered_addr))
    }

    async fn register_or_get_web3_user(
        &self,
        user_grpc_client: &UserServiceClient<Channel>,
        chain_id: i64,
        address: String,
    ) -> Result<i64, AppError> {
        use common_proto::user::Web3InfoReq;

        // 1. 尝试获取用户信息
        let web3_info_req = tonic::Request::new(Web3InfoReq {
            chain_id,
            address: address.clone(),
        });

        let user_info_result = user_grpc_client
            .clone()
            .get_user_info_by_web3(web3_info_req)
            .await;

        // 2. 如果用户存在，返回 user_id；否则自动注册
        match user_info_result {
            Ok(response) => {
                let user_info = response.into_inner();
                tracing::debug!("Web3 user found: user_id={}", user_info.id);
                Ok(user_info.id)
            }
            Err(_) => {
                // 用户不存在，自动注册
                tracing::info!(
                    "Web3 user not found, auto-registering: chain_id={}, address={}",
                    chain_id,
                    &address
                );

                let register_req = tonic::Request::new(RegisterUserReq {
                    register_type: RegisterType::Web3 as i32,
                    username: None,
                    password: None,
                    web3_address: Some(address.clone()),
                    web3_chain_id: Some(chain_id),
                    email: None,
                });

                let register_res = user_grpc_client
                    .clone()
                    .register_user(register_req)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to register user: {}", e)))?
                    .into_inner();

                tracing::info!(
                    "Web3 user auto-registered successfully: user_id={}",
                    register_res.user_id
                );
                Ok(register_res.user_id)
            }
        }
    }
}
