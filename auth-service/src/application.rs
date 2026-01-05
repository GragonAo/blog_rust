use common_redis::application::Redis;

pub struct Snowflake {
    pub machine_id: i32,
    pub node_id: i32,
}

pub struct JWT {
    pub secret: String,
    pub expiration_hours: u64,
    pub refresh_expiration_hours: u64,
}

pub struct AppConfig {
    pub redis: Redis,
    pub snowflake: Snowflake,
    pub jwt: JWT,
    pub bind_addr: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            redis: Redis {
                host: "192.168.31.218".into(),
                port: 6379,
                password: Some("redis123456".into()),
                db: 0,
                pool_size: 4,
            },
            snowflake: Snowflake {
                machine_id: 1,
                node_id: 1,
            },
            jwt: JWT {
                secret: "your_secret_key".into(),
                expiration_hours: 24,
                refresh_expiration_hours: 24 * 7,
            },
            bind_addr: "0.0.0.0:5010".into(),
        }
    }
}
