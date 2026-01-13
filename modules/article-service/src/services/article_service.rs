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
    domain::bo::authorship_bo::AuthorshipBo,
    domain::{
        bo::article_bo::{ArticleDetailBo, ArticleQuery},
        model::article::{ArticleDetail, ArticleStatus, ArticleSummary},
    },
    repository::{
        article_repository::{ArticleRepository, ArticleRepositoryImpl},
        authorship_repository::{AuthorshipRepository, AuthorshipRepositoryImpl},
    },
    with_transaction,
};

static ARTICLE_REPO: ArticleRepositoryImpl = ArticleRepositoryImpl;
static AUTHORSHIP_REPO: AuthorshipRepositoryImpl = AuthorshipRepositoryImpl;

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

    /// 删除文章
    async fn delete(&self, article_id: i64) -> Result<(), AppError>;

    // 浏览文章（增加浏览数和作者统计）
    async fn view_article(&self, article_id: i64) -> Result<Option<ArticleDetail>, AppError>;
    // 点赞文章（增加点赞数和作者统计）
    async fn like_article(&self, article_id: i64) -> Result<(), AppError>;
    // 收藏文章（增加收藏数和作者统计）
    async fn collect_article(&self, article_id: i64) -> Result<(), AppError>;

    /// 更新文章
    async fn update(&self, article_detail_bo: ArticleDetailBo) -> Result<(), AppError>;

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

        // 使用事务确保文章插入和作者统计更新的原子性
        with_transaction!(&self.db_pool, |tx| async {
            // 插入文章
            ARTICLE_REPO.insert(tx, &article).await?;

            // 更新作者的文章数
            let authorship_bo = AuthorshipBo {
                uid: article.uid,
                like_count: None,
                fellow_count: None,
                collect_count: None,
                article_count_public: Some(1),
                article_count_private: None,
            };
            AUTHORSHIP_REPO.upsert(tx, &authorship_bo).await?;

            Ok(article_id)
        })
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
        // 使用事务确保删除和统计更新的原子性
        with_transaction!(&self.db_pool, |tx| async {
            // 获取文章信息，用于后续更新作者数据
            let article = ARTICLE_REPO.find_by_id(tx, article_id).await?;

            // 执行删除
            ARTICLE_REPO.delete_by_id(tx, article_id).await?;

            // 如果文章存在，更新作者的文章数（减少）
            if let Some(article) = article {
                let authorship_bo = AuthorshipBo {
                    uid: article.uid,
                    like_count: None,
                    fellow_count: None,
                    collect_count: None,
                    article_count_public: Some(-1),
                    article_count_private: None,
                };
                AUTHORSHIP_REPO.upsert(tx, &authorship_bo).await?;
            }

            Ok(())
        })
    }

    async fn view_article(&self, article_id: i64) -> Result<Option<ArticleDetail>, AppError> {
        // 先获取文章详情
        let article = self.get_article_details(article_id).await?;

        if let Some(ref a) = article {
            // 使用事务同时更新文章浏览数和作者统计
            with_transaction!(&self.db_pool, |tx| async {
                ARTICLE_REPO.update_views(tx, a.id, 1).await?;

                let authorship_bo = AuthorshipBo {
                    uid: a.uid,
                    like_count: None,
                    fellow_count: Some(1), // 浏览数统计
                    collect_count: None,
                    article_count_public: None,
                    article_count_private: None,
                };
                AUTHORSHIP_REPO.upsert(tx, &authorship_bo).await?;

                Ok(())
            })?;
        }

        Ok(article)
    }

    async fn like_article(&self, article_id: i64) -> Result<(), AppError> {
        // 获取文章信息
        let article = self
            .get_article_details(article_id)
            .await?
            .ok_or_else(|| AppError::db("Article not found"))?;

        // 使用事务同时更新文章点赞数和作者统计
        with_transaction!(&self.db_pool, |tx| async {
            ARTICLE_REPO.update_likes(tx, article.id, 1).await?;

            let authorship_bo = AuthorshipBo {
                uid: article.uid,
                like_count: Some(1),
                fellow_count: None,
                collect_count: None,
                article_count_public: None,
                article_count_private: None,
            };
            AUTHORSHIP_REPO.upsert(tx, &authorship_bo).await?;

            Ok(())
        })
    }

    async fn collect_article(&self, article_id: i64) -> Result<(), AppError> {
        // 获取文章信息
        let article = self
            .get_article_details(article_id)
            .await?
            .ok_or_else(|| AppError::db("Article not found"))?;

        // 使用事务同时更新文章收藏数和作者统计
        with_transaction!(&self.db_pool, |tx| async {
            ARTICLE_REPO.update_collects(tx, article.id, 1).await?;

            let authorship_bo = AuthorshipBo {
                uid: article.uid,
                like_count: None,
                fellow_count: None,
                collect_count: Some(1),
                article_count_public: None,
                article_count_private: None,
            };
            AUTHORSHIP_REPO.upsert(tx, &authorship_bo).await?;

            Ok(())
        })
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
