use async_trait::async_trait;
use common_core::AppError;
use common_redis::RedisClient;
use common_web3::{Web3Recover, chain::Chain};
use snowflake::SnowflakeIdGenerator;
use std::sync::{Arc, Mutex};

const LOGIN_WEB3_NONCE_CACHE: &str = "blog:auth:login:web3:nonce";
const NONCE_EXPIRATION_SECONDS: u64 = 300;

#[async_trait]
pub trait LoginService: Send + Sync {
    async fn get_login_web3_nonce(&self, chain_id: u64) -> Result<String, AppError>;
    async fn login_web3_wallet(
        &self,
        signature: String,
        message: String,
    ) -> Result<String, AppError>;
}

pub struct LoginServiceImpl {
    pub redis_client: Arc<RedisClient>,
    pub db_pool: String,
    pub id_generator: std::sync::Arc<Mutex<SnowflakeIdGenerator>>,
}

#[async_trait]
impl LoginService for LoginServiceImpl {
    async fn get_login_web3_nonce(&self, chain_id: u64) -> Result<String, AppError> {
        let nonce_id = {
            let mut id_gen = self
                .id_generator
                .lock()
                .map_err(|_| AppError::Internal("Generator lock poisoned".into()))?;
            id_gen.generate().to_string()
        };

        let redis_key = format!("{}:{}", LOGIN_WEB3_NONCE_CACHE, nonce_id);
        let value = chain_id.to_string();
        self.redis_client
            .set_ex(&redis_key, &value, NONCE_EXPIRATION_SECONDS)
            .await?;

        Ok(nonce_id)
    }

    async fn login_web3_wallet(
        &self,
        signature: String,
        message: String,
    ) -> Result<String, AppError> {
        let nonce = message
            .split(": ")
            .nth(1)
            .ok_or_else(|| AppError::Internal("Invalid message format".into()))?;
        let redis_key = format!("{}:{}", LOGIN_WEB3_NONCE_CACHE, nonce);
        let chain_id_str = self
            .redis_client
            .get_str(&redis_key)
            .await?
            .ok_or_else(|| AppError::Internal("Nonce expired or invalid".into()))?;
        self.redis_client.del(&redis_key).await?;

        let chain_id = chain_id_str
            .parse::<u64>()
            .map_err(|_| AppError::Internal("Invalid chain ID stored in nonce".into()))?;
        let chain = Chain::try_from(chain_id)?;
        // 调用底层：换出地址
        let recovered_addr = Web3Recover::get_address(chain, &message, &signature)?;
        Ok(recovered_addr)
    }
}
