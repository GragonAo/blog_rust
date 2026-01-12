use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub title_like: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleDetailBo {
    pub id: Option<i64>,
    pub uid: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub cover_urls: Option<Vec<String>>,
}
