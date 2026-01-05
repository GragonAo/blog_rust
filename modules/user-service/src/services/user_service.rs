use crate::domain::model::user::User;
use crate::repository::user_repository::{UserRepository, UserRepositoryImpl};
use async_trait::async_trait;
use chrono::Utc;
use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user_info(&self, user_id: u64) -> Result<Option<User>, AppError>;
    async fn get_user_info_by_web3(
        &self,
        chain_id: u64,
        web3_address: String,
    ) -> Result<(), AppError>;
    async fn create_user(&self, username: String, email: Option<String>) -> Result<User, AppError>;
}

pub struct UserServiceImpl {
    pub redis_client: RedisClient,
    pub db_pool: PgPool,
    pub id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    /// 获取用户信息
    async fn get_user_info(&self, user_id: u64) -> Result<Option<User>, AppError> {
        let repo = UserRepositoryImpl::new(self.db_pool.clone());
        let user = repo.find_by_id(user_id as i64).await?;
        Ok(user)
    }

    async fn get_user_info_by_web3(
        &self,
        chain_id: u64,
        web3_address: String,
    ) -> Result<(), AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u 
             INNER JOIN user_web3_info w ON u.id = w.user_id 
             WHERE w.chain_id = $1 AND w.web3_address = $2",
        )
        .bind(chain_id as i64)
        .bind(web3_address)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| AppError::Db(format!("Failed to fetch web3 user: {}", e)))?;

        if let Some(_user) = user {
            // 处理用户信息
        }

        Ok(())
    }

    /// 创建用户
    async fn create_user(&self, username: String, email: Option<String>) -> Result<User, AppError> {
        // 生成用户 ID
        let user_id = self.id_generator.lock().unwrap().real_time_generate();

        // 创建用户对象
        let user = User {
            id: user_id,
            username,
            email,
            password_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repo = UserRepositoryImpl::new(self.db_pool.clone());
        repo.create(&user).await?;

        Ok(user)
    }
}
