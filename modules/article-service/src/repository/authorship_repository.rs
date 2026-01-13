use async_trait::async_trait;
use common_core::AppError;
use sqlx::PgConnection;

use crate::domain::{bo::authorship_bo::AuthorshipBo, model::authorship::Authorship};

#[async_trait]
pub trait AuthorshipRepository: Send + Sync {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        uid: i64,
    ) -> Result<Option<Authorship>, AppError>;

    async fn insert(
        &self,
        executor: &mut PgConnection,
        authorship: &Authorship,
    ) -> Result<(), AppError>;

    /// 更新或插入（Upsert）作者信息，如果不存在则创建默认记录
    async fn upsert(
        &self,
        executor: &mut PgConnection,
        authorship_bo: &AuthorshipBo,
    ) -> Result<(), AppError>;
}

pub struct AuthorshipRepositoryImpl;
#[async_trait]
impl AuthorshipRepository for AuthorshipRepositoryImpl {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        uid: i64,
    ) -> Result<Option<Authorship>, AppError> {
        sqlx::query_as::<_, Authorship>(
            "SELECT uid, like_count, fellow_count, collect_count, article_count_public, article_count_private \
             FROM authorship WHERE uid = $1",
        )
        .bind(uid)
        .fetch_optional(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to find authorship by uid: {}", e)))
    }

    async fn insert(
        &self,
        executor: &mut PgConnection,
        authorship: &Authorship,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO authorship (
                uid, like_count, fellow_count, collect_count, 
                article_count_public, article_count_private
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(authorship.uid)
        .bind(authorship.like_count)
        .bind(authorship.fellow_count)
        .bind(authorship.collect_count)
        .bind(authorship.article_count_public)
        .bind(authorship.article_count_private)
        .execute(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to insert authorship: {}", e)))?;
        Ok(())
    }

    async fn upsert(
        &self,
        executor: &mut PgConnection,
        authorship_bo: &AuthorshipBo,
    ) -> Result<(), AppError> {
        // 使用 PostgreSQL 的 ON CONFLICT 进行 upsert 操作
        let mut updates = Vec::new();

        // 构建更新语句
        if let Some(like_count) = authorship_bo.like_count {
            if like_count >= 0 {
                updates.push(format!(
                    "like_count = authorship.like_count + {}",
                    like_count
                ));
            } else {
                updates.push(format!(
                    "like_count = GREATEST(0, authorship.like_count + {})",
                    like_count
                ));
            }
        }
        if let Some(fellow_count) = authorship_bo.fellow_count {
            if fellow_count >= 0 {
                updates.push(format!(
                    "fellow_count = authorship.fellow_count + {}",
                    fellow_count
                ));
            } else {
                updates.push(format!(
                    "fellow_count = GREATEST(0, authorship.fellow_count + {})",
                    fellow_count
                ));
            }
        }
        if let Some(collect_count) = authorship_bo.collect_count {
            if collect_count >= 0 {
                updates.push(format!(
                    "collect_count = authorship.collect_count + {}",
                    collect_count
                ));
            } else {
                updates.push(format!(
                    "collect_count = GREATEST(0, authorship.collect_count + {})",
                    collect_count
                ));
            }
        }
        if let Some(article_count_public) = authorship_bo.article_count_public {
            if article_count_public >= 0 {
                updates.push(format!(
                    "article_count_public = authorship.article_count_public + {}",
                    article_count_public
                ));
            } else {
                updates.push(format!(
                    "article_count_public = GREATEST(0, authorship.article_count_public + {})",
                    article_count_public
                ));
            }
        }
        if let Some(article_count_private) = authorship_bo.article_count_private {
            if article_count_private >= 0 {
                updates.push(format!(
                    "article_count_private = authorship.article_count_private + {}",
                    article_count_private
                ));
            } else {
                updates.push(format!(
                    "article_count_private = GREATEST(0, authorship.article_count_private + {})",
                    article_count_private
                ));
            }
        }

        let update_clause = if updates.is_empty() {
            // 如果没有更新字段，仍然执行UPSERT但不更新任何值
            "uid = EXCLUDED.uid".to_string()
        } else {
            updates.join(", ")
        };

        let query = format!(
            r#"
            INSERT INTO authorship (uid, like_count, fellow_count, collect_count, article_count_public, article_count_private)
            VALUES ($1, 0, 0, 0, 0, 0)
            ON CONFLICT (uid)
            DO UPDATE SET {}
            "#,
            update_clause
        );

        sqlx::query(&query)
            .bind(authorship_bo.uid)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to upsert authorship: {}", e)))?;

        Ok(())
    }
}
