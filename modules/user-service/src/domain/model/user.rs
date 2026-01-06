use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Web3 用户信息数据库模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Web3UserInfo {
    pub id: i64,
    pub user_id: i64,
    pub chain_id: i64,
    pub address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户完整信息（包含 web3 信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(flatten)]
    pub user: User,
    pub web3_info: Option<Web3UserInfo>,
}
