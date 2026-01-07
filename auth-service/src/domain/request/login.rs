use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginWeb3Request {
    pub signature: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct LoginWeb3NonceQuery {
    pub chain_id: i64,
    pub address: String,
}
