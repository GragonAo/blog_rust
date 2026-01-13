use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AuthorshipBo {
    pub uid: i64,
    pub like_count: Option<i32>,
    pub fellow_count: Option<i32>,
    pub collect_count: Option<i32>,
    pub article_count_public: Option<i32>,
    pub article_count_private: Option<i32>,
}
