use std::sync::Arc;

use async_trait::async_trait;
use common_core::AppError;
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{
    domain::model::authorship::Authorship,
    repository::authorship_repository::{AuthorshipRepository, AuthorshipRepositoryImpl},
};

static AUTHORSHIP_REPO: AuthorshipRepositoryImpl = AuthorshipRepositoryImpl;

/// 作者信息服务
#[async_trait]
pub trait AuthorshipService: Send + Sync {
    /// 获取作者信息，如果不存在会自动创建
    async fn get_authorship(&self, uid: i64) -> Result<Authorship, AppError>;
}

pub struct AuthorshipServiceImpl {
    #[allow(dead_code)]
    pub redis_client: RedisClient,
    pub db_pool: PgPool,
    #[allow(dead_code)]
    pub id_generator: Arc<RwLock<SnowflakeIdGenerator>>,
}

#[async_trait]
impl AuthorshipService for AuthorshipServiceImpl {
    async fn get_authorship(&self, uid: i64) -> Result<Authorship, AppError> {
        self._ensure_authorship(uid).await
    }
}

// 私有辅助方法
impl AuthorshipServiceImpl {
    /// 私有方法：确保作者信息存在的内部工具方法
    async fn _ensure_authorship(&self, uid: i64) -> Result<Authorship, AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;

        // 先尝试查找
        if let Some(authorship) = AUTHORSHIP_REPO.find_by_id(&mut conn, uid).await? {
            return Ok(authorship);
        }

        // 不存在则创建默认记录
        let default_authorship = Authorship {
            uid,
            like_count: 0,
            fellow_count: 0,
            collect_count: 0,
            article_count_public: 0,
            article_count_private: 0,
        };

        AUTHORSHIP_REPO
            .insert(&mut conn, &default_authorship)
            .await?;
        Ok(default_authorship)
    }
}
