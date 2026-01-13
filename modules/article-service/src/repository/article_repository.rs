use async_trait::async_trait;
use common_core::AppError;
use sqlx::PgConnection;

use crate::domain::{
    bo::article_bo::ArticleQuery,
    model::article::{ArticleDetail, ArticleSummary},
};

#[async_trait]
pub trait ArticleRepository: Send + Sync {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        id: i64,
    ) -> Result<Option<ArticleDetail>, AppError>;

    async fn find_list(
        &self,
        executor: &mut PgConnection,
        limit: i64,
        offset: i64,
        query: &ArticleQuery,
    ) -> Result<Vec<ArticleSummary>, AppError>;

    async fn count(
        &self,
        executor: &mut PgConnection,
        query: &ArticleQuery,
    ) -> Result<i64, AppError>;

    async fn insert(
        &self,
        executor: &mut PgConnection,
        article: &ArticleDetail,
    ) -> Result<(), AppError>;

    async fn update(
        &self,
        executor: &mut PgConnection,
        id: i64,
        title: Option<String>,
        description: Option<String>,
        content: Option<String>,
        cover_urls: Option<Vec<String>>,
    ) -> Result<(), AppError>;

    async fn delete_by_id(&self, executor: &mut PgConnection, id: i64) -> Result<(), AppError>;

    async fn is_owner(
        &self,
        executor: &mut PgConnection,
        id: i64,
        uid: i64,
    ) -> Result<bool, AppError>;

    async fn update_likes(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError>;

    async fn update_views(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError>;

    async fn update_collects(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError>;
}

pub struct ArticleRepositoryImpl;

#[async_trait]
impl ArticleRepository for ArticleRepositoryImpl {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        id: i64,
    ) -> Result<Option<ArticleDetail>, AppError> {
        sqlx::query_as::<_, ArticleDetail>(
            "SELECT * FROM articles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to find article by id: {}", e)))
    }

    async fn find_list(
        &self,
        executor: &mut PgConnection,
        limit: i64,
        offset: i64,
        query: &ArticleQuery,
    ) -> Result<Vec<ArticleSummary>, AppError> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, uid, title, description, status, likes, views, collects, cover_urls, created_at, updated_at \
         FROM articles WHERE deleted_at IS NULL",
        );

        if let Some(title) = &query.title_like {
            query_builder.push(" AND title LIKE ");
            query_builder.push_bind(format!("%{}%", title));
        }

        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        query_builder
            .build_query_as::<ArticleSummary>()
            .fetch_all(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to find article list: {}", e)))
    }

    async fn count(
        &self,
        executor: &mut PgConnection,
        query: &ArticleQuery,
    ) -> Result<i64, AppError> {
        let mut query_builder =
            sqlx::QueryBuilder::new("SELECT COUNT(*) FROM articles WHERE deleted_at IS NULL");

        if let Some(title) = &query.title_like {
            query_builder.push(" AND title LIKE ");
            query_builder.push_bind(format!("%{}%", title));
        }

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to count articles: {}", e)))?;
        Ok(count.0)
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        article: &ArticleDetail,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO articles (
                id, uid, title, description, content, status, 
                likes, views, collects, cover_urls, 
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(article.id)
        .bind(article.uid)
        .bind(&article.title)
        .bind(&article.description)
        .bind(&article.content)
        .bind(article.status)
        .bind(article.likes)
        .bind(article.views)
        .bind(article.collects)
        .bind(&article.cover_urls)
        .bind(article.created_at)
        .bind(article.updated_at)
        .execute(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to insert article: {}", e)))?;
        Ok(())
    }

    async fn update(
        &self,
        executor: &mut PgConnection,
        id: i64,
        title: Option<String>,
        description: Option<String>,
        content: Option<String>,
        cover_urls: Option<Vec<String>>,
    ) -> Result<(), AppError> {
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE articles SET updated_at = NOW()");

        if let Some(t) = title {
            query_builder.push(", title = ");
            query_builder.push_bind(t);
        }
        if let Some(d) = description {
            query_builder.push(", description = ");
            query_builder.push_bind(d);
        }
        if let Some(c) = content {
            query_builder.push(", content = ");
            query_builder.push_bind(c);
        }
        if let Some(urls) = cover_urls {
            query_builder.push(", cover_urls = ");
            query_builder.push_bind(urls);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" AND deleted_at IS NULL");

        query_builder
            .build()
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to update article: {}", e)))?;
        Ok(())
    }

    async fn delete_by_id(&self, executor: &mut PgConnection, id: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE articles SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to delete article: {}", e)))?;
        Ok(())
    }

    async fn is_owner(
        &self,
        executor: &mut PgConnection,
        id: i64,
        uid: i64,
    ) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM articles WHERE id = $1 AND uid = $2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(uid)
        .fetch_one(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to check article ownership: {}", e)))?;
        Ok(count.0 > 0)
    }

    async fn update_likes(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError> {
        let sql = if increment >= 0 {
            "UPDATE articles SET likes = likes + $1, updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        } else {
            "UPDATE articles SET likes = GREATEST(0, likes + $1), updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        };
        
        sqlx::query(sql)
            .bind(increment)
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to update article likes: {}", e)))?;
        Ok(())
    }

    async fn update_views(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError> {
        let sql = if increment >= 0 {
            "UPDATE articles SET views = views + $1, updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        } else {
            "UPDATE articles SET views = GREATEST(0, views + $1), updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        };
        
        sqlx::query(sql)
            .bind(increment)
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to update article views: {}", e)))?;
        Ok(())
    }

    async fn update_collects(
        &self,
        executor: &mut PgConnection,
        id: i64,
        increment: i32,
    ) -> Result<(), AppError> {
        let sql = if increment >= 0 {
            "UPDATE articles SET collects = collects + $1, updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        } else {
            "UPDATE articles SET collects = GREATEST(0, collects + $1), updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL"
        };
        
        sqlx::query(sql)
            .bind(increment)
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to update article collects: {}", e)))?;
        Ok(())
    }
}
