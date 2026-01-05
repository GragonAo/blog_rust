use common_redis::application::Redis;

pub struct AppConfig {
    pub redis: Redis,
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
            bind_addr: "0.0.0.0:5010".into(),
        }
    }
}
