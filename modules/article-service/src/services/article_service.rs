use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use common_core::{
    AppError,
    domain::page::{Page, PageResult},
};
use common_redis::RedisClient;
use snowflake::SnowflakeIdGenerator;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{
    domain::{
        bo::article_bo::{ArticleDetailBo, ArticleQuery},
        model::article::{ArticleDetail, ArticleStatus, ArticleSummary},
    },
    repository::article_repository::{ArticleRepository, ArticleRepositoryImpl},
};

static ARTICLE_REPO: ArticleRepositoryImpl = ArticleRepositoryImpl;

/// 文章服务
#[async_trait]
pub trait ArticleService: Send + Sync {
    /// 获取文章详情
    async fn get_article_details(&self, article_id: i64)
    -> Result<Option<ArticleDetail>, AppError>;

    /// 获取文章列表
    async fn get_article_list(
        &self,
        query: ArticleQuery,
        page: Page,
    ) -> Result<PageResult<ArticleSummary>, AppError>;

    /// 插入文章
    async fn insert(&self, article_detail_bo: ArticleDetailBo) -> Result<i64, AppError>;

    /// 更新文章
    async fn update(&self, article_detail_bo: ArticleDetailBo) -> Result<(), AppError>;

    /// 删除文章
    async fn delete(&self, article_id: i64) -> Result<(), AppError>;

    /// 检查文章所有权
    async fn check_ownership(&self, article_id: i64, uid: i64) -> Result<bool, AppError>;
}

pub struct ArticleServiceImpl {
    #[allow(dead_code)]
    pub redis_client: RedisClient,
    pub db_pool: PgPool,
    pub id_generator: Arc<RwLock<SnowflakeIdGenerator>>,
}

#[async_trait]
impl ArticleService for ArticleServiceImpl {
    async fn get_article_details(
        &self,
        article_id: i64,
    ) -> Result<Option<ArticleDetail>, AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;
        ARTICLE_REPO.find_by_id(&mut conn, article_id).await
    }

    async fn get_article_list(
        &self,
        query: ArticleQuery,
        page: Page,
    ) -> Result<PageResult<ArticleSummary>, AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;

        // 分页参数
        let limit = page.page_size;
        let offset = (page.page_num - 1) * limit;

        // 查询总数
        let total = ARTICLE_REPO.count(&mut conn, &query).await?;

        // 查询列表
        let items = if total > 0 {
            ARTICLE_REPO
                .find_list(&mut conn, limit, offset, &query)
                .await?
        } else {
            Vec::new()
        };

        Ok(PageResult {
            total,
            list: items,
            page_num: page.page_num,
            page_size: page.page_size,
        })
    }

    async fn insert(&self, article_detail_bo: ArticleDetailBo) -> Result<i64, AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;
        let article_id = self.id_generator.write().await.real_time_generate();
        let now = Utc::now();

        let article = ArticleDetail {
            id: article_id,
            uid: article_detail_bo.uid,
            title: article_detail_bo
                .title
                .ok_or(AppError::internal("Title is required"))?,
            description: article_detail_bo
                .description
                .ok_or(AppError::internal("Description is required"))?,
            content: article_detail_bo
                .content
                .ok_or(AppError::internal("Content is required"))?,
            status: ArticleStatus::Draft,
            likes: 0,
            views: 0,
            collects: 0,
            cover_urls: article_detail_bo.cover_urls.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        ARTICLE_REPO.insert(&mut conn, &article).await?;

        Ok(article_id)
    }

    async fn update(&self, article_detail_bo: ArticleDetailBo) -> Result<(), AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;

        // 只有当有需要更新的字段时才执行更新
        if article_detail_bo.title.is_none()
            && article_detail_bo.description.is_none()
            && article_detail_bo.content.is_none()
            && article_detail_bo.cover_urls.is_none()
        {
            return Ok(());
        }

        // 确保 ID 存在
        let id = article_detail_bo
            .id
            .ok_or(AppError::db("Article ID is required for update"))?;

        ARTICLE_REPO
            .update(
                &mut conn,
                id,
                article_detail_bo.title,
                article_detail_bo.description,
                article_detail_bo.content,
                article_detail_bo.cover_urls,
            )
            .await
    }

    async fn delete(&self, article_id: i64) -> Result<(), AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;
        ARTICLE_REPO.delete_by_id(&mut conn, article_id).await
    }

    async fn check_ownership(&self, article_id: i64, uid: i64) -> Result<bool, AppError> {
        let mut conn = self
            .db_pool
            .acquire()
            .await
            .map_err(|e| AppError::db(e.to_string()))?;
        ARTICLE_REPO.is_owner(&mut conn, article_id, uid).await
    }
}
