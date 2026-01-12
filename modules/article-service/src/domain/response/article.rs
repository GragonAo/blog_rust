use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::model::article::{ArticleDetail, ArticleStatus, ArticleSummary};

#[derive(Clone, Serialize, Deserialize)]
pub struct ArticleSummaryRes {
    pub id: i64,
    pub uid: i64,
    pub title: String,
    pub description: String,
    pub status: ArticleStatus,
    pub likes: i64,
    pub views: i64,
    pub collects: i64,
    pub cover_urls: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ArticleSummary> for ArticleSummaryRes {
    fn from(summary: ArticleSummary) -> Self {
        Self {
            id: summary.id,
            uid: summary.uid,
            title: summary.title,
            description: summary.description,
            status: summary.status,
            likes: summary.likes,
            views: summary.views,
            collects: summary.collects,
            cover_urls: summary.cover_urls,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArticleDetailRes {
    pub id: i64,
    pub uid: i64,
    pub title: String,
    pub description: String,
    pub content: String,
    pub status: ArticleStatus,
    pub likes: i64,
    pub views: i64,
    pub collects: i64,
    pub cover_urls: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<ArticleDetail> for ArticleDetailRes {
    fn from(detail: ArticleDetail) -> Self {
        Self {
            id: detail.id,
            uid: detail.uid,
            title: detail.title,
            description: detail.description,
            content: detail.content,
            status: detail.status,
            likes: detail.likes,
            views: detail.views,
            collects: detail.collects,
            cover_urls: detail.cover_urls,
            created_at: detail.created_at,
            updated_at: detail.updated_at,
            deleted_at: detail.deleted_at,
        }
    }
}
