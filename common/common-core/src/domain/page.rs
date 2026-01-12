use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Page {
    pub page_num: i64,
    pub page_size: i64,
}

#[derive(Serialize)]
pub struct PageResult<T> {
    pub list: Vec<T>,
    pub total: i64,
    pub page_num: i64,
    pub page_size: i64,
}
