use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户传输模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserBo {
    pub id: Option<i64>,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

/// Web3 用户信息传输模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Web3UserInfoBo {
    pub chain_id: i64,
    pub address: String,
}

/// 用户完整信息（包含 web3 信息）传输模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoBo {
    #[serde(flatten)]
    pub user: UserBo,
    pub web3_info: Option<Web3UserInfoBo>,
}
