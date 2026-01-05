use serde::Serialize;

#[derive(Serialize)]
pub struct Web3UserInfo {
    pub web3_address: String,
    pub web3_chain: String,
}

#[derive(Serialize)]
pub struct UserInfoResponse {
    pub user_id: i64,
    pub username: String,
    pub web3_user_info: Option<Web3UserInfo>,
}
