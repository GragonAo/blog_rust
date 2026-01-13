use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthorshipRes {
    pub id: String,
    pub name: String,
    pub avatar_url: String,
    pub like_count: i32,
    pub fellow_count: i32,
    pub collect_count: i32,
    pub article_count: i32,
}
