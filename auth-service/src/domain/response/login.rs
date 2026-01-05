use serde::Serialize;

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub expire_in: u64,
    pub refresh_token: String,
    pub refresh_expire_in: u64,
    pub client_id: String,
}
