use serde::Serialize;

#[derive(Serialize)]
pub struct Web3UserInfo {
    pub address: String,
    pub chain_id: i64,
}

#[derive(Serialize)]
pub struct UserInfoResponse {
    pub user_id: i64,
    pub username: String,
    pub web3_user_info: Option<Web3UserInfo>,
}

impl From<crate::domain::model::user::UserInfo> for UserInfoResponse {
    fn from(ui: crate::domain::model::user::UserInfo) -> Self {
        Self {
            user_id: ui.user.id,
            username: ui.user.username,
            web3_user_info: ui.web3_info.map(|w| Web3UserInfo {
                chain_id: w.chain_id,
                address: w.address,
            }),
        }
    }
}
