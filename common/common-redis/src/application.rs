use serde::Deserialize;

fn default_db() -> u8 {
    0
}

fn default_pool_size() -> u32 {
    8
}

#[derive(Debug, Clone, Deserialize)]
pub struct Redis {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_db")]
    pub db: u8,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

impl Redis {
    pub fn url(&self) -> String {
        let auth = self
            .password
            .as_ref()
            .map(|pwd| format!(":{pwd}@"))
            .unwrap_or_default();

        format!("redis://{auth}{}:{}/{}", self.host, self.port, self.db)
    }
}
