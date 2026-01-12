use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct ArticleDetail {
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Type, PartialEq)]
#[repr(i32)]
pub enum ArticleStatus {
    /// 公开
    Public = 1,
    /// 私有
    Private = 2,
    /// 草稿
    Draft = 3,
}

#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct ArticleSummary {
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
