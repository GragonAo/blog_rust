use crate::domain::model::user::Web3UserInfo;
use async_trait::async_trait;
use common_core::AppError;
use sqlx::PgConnection;

#[async_trait]
pub trait Web3UserRepository: Send + Sync {
    /// 根据用户 ID 获取 Web3 信息
    async fn find_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i64,
    ) -> Result<Option<Web3UserInfo>, AppError>;

    /// 插入 Web3 信息
    async fn insert(
        &self,
        executor: &mut PgConnection,
        info: &Web3UserInfo,
    ) -> Result<(), AppError>;

    /// 根据地址和链 ID 查询
    async fn find_by_address(
        &self,
        executor: &mut PgConnection,
        chain_id: i64,
        address: &str,
    ) -> Result<Option<Web3UserInfo>, AppError>;
}

pub struct Web3UserRepositoryImpl;

impl Web3UserRepositoryImpl {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Web3UserRepository for Web3UserRepositoryImpl {
    async fn find_by_user_id(
        &self,
        executor: &mut PgConnection,
        user_id: i64,
    ) -> Result<Option<Web3UserInfo>, AppError> {
        sqlx::query_as::<_, Web3UserInfo>("SELECT * FROM web3_user_info WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch web3 info: {}", e)))
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        info: &Web3UserInfo,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO web3_user_info (id, user_id, chain_id, address, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(info.id)
        .bind(info.user_id)
        .bind(info.chain_id)
        .bind(&info.address)
        .bind(info.created_at)
        .bind(info.updated_at)
        .execute(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to save web3 info: {}", e)))?;
        Ok(())
    }

    async fn find_by_address(
        &self,
        executor: &mut PgConnection,
        chain_id: i64,
        address: &str,
    ) -> Result<Option<Web3UserInfo>, AppError> {
        sqlx::query_as::<_, Web3UserInfo>(
            "SELECT * FROM web3_user_info WHERE chain_id = $1 AND address = $2",
        )
        .bind(chain_id)
        .bind(address)
        .fetch_optional(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to fetch web3 info by address: {}", e)))
    }
}
