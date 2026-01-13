use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, FromRow, Serialize, Deserialize)]
pub struct Authorship {
    pub uid: i64,
    pub like_count: i32,
    pub fellow_count: i32,
    pub collect_count: i32,
    pub article_count_public: i32,
    pub article_count_private: i32,
}
