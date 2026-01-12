use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub name: String,
    pub bind_addr: String,
    pub grpc_addr: Option<String>,
}
