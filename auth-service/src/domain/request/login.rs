use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginWeb3Request {
    pub address: String,
    pub signature: String,
    pub message: String,
}
