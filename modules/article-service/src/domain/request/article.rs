use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArticleRequest {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub cover_urls: Option<Vec<String>>,
}
