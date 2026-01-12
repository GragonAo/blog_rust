use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Logs {
    pub path: String,
}
