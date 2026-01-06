use crate::domain::bo::user_bo::UserInfoBo;
use crate::domain::model::user::{User, UserInfo, Web3UserInfo};
use crate::repository::user_repository::{UserRepository, UserRepositoryImpl};
use crate::repository::web3_user_info_repository::{Web3UserRepository, Web3UserRepositoryImpl};
use async_trait::async_trait;
use chrono::Utc;
use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user_info(&self, user_id: i64) -> Result<Option<UserInfo>, AppError>;
    async fn get_user_info_by_web3(
        &self,
        chain_id: i64,
        web3_address: String,
    ) -> Result<(), AppError>;
    async fn create_user(&self, user_info_bo: UserInfoBo) -> Result<i64, AppError>;
}

pub struct UserServiceImpl {
    #[allow(dead_code)]
    pub redis_client: RedisClient,
    pub db_pool: PgPool,
    pub id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    /// 获取用户信息
    async fn get_user_info(&self, user_id: i64) -> Result<Option<UserInfo>, AppError> {
        // 1. 获取一个独占连接
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::Db(e.to_string()))?;

        // 2. 初始化 Repos
        let user_repo = UserRepositoryImpl::new(self.db_pool.clone());
        let web3_repo = Web3UserRepositoryImpl::new(self.db_pool.clone());

        let user_opt = user_repo.find_by_id(&mut *conn, user_id).await?;

        if let Some(user) = user_opt {
            let web3_info = web3_repo.find_by_user_id(&mut *conn, user_id).await?;
            Ok(Some(UserInfo { user, web3_info }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_info_by_web3(
        &self,
        _chain_id: i64,
        _web3_address: String,
    ) -> Result<(), AppError> {
        Ok(())
    }

    /// 创建用户
    async fn create_user(&self, user_info_bo: UserInfoBo) -> Result<i64, AppError> {
        let user_id = self.id_generator.lock().unwrap().real_time_generate();
        let user_info = UserInfo {
            user: User {
                id: user_id,
                username: user_info_bo.user.username,
                email: user_info_bo.user.email,
                password_hash: user_info_bo.user.password_hash,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            web3_info: user_info_bo.web3_info.map(|w| Web3UserInfo {
                id: self.id_generator.lock().unwrap().real_time_generate(),
                user_id: user_id,
                chain_id: w.chain_id,
                address: w.address,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }),
        };

        // 3. 开启事务
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| AppError::Db(format!("Begin transaction failed: {}", e)))?;

        let repo = UserRepositoryImpl::new(self.db_pool.clone());

        // 4. 插入用户表
        repo.inster(&mut *tx, &user_info.user).await?;

        // 5. 处理 Web3 信息
        if let Some(mut web3) = user_info.web3_info {
            web3.user_id = user_id;
            web3.created_at = Utc::now();
            web3.updated_at = Utc::now();

            let web3_repo = Web3UserRepositoryImpl::new(self.db_pool.clone());
            web3_repo.insert(&mut *tx, &web3).await?;
        }
        // 6. 提交事务
        tx.commit()
            .await
            .map_err(|e| AppError::Db(format!("Commit transaction failed: {}", e)))?;

        Ok(user_id)
    }
}
