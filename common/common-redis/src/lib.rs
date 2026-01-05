use bb8_redis::{
    RedisConnectionManager,
    bb8::{Pool, PooledConnection},
};
use common_core::{AppError, AppResult};
use redis::{AsyncCommands, cmd};

pub mod application;

pub type ConnectionPool = Pool<RedisConnectionManager>;
pub type Connection<'a> = PooledConnection<'a, RedisConnectionManager>;

#[derive(Clone)]
pub struct RedisClient {
    pool: ConnectionPool,
}

impl RedisClient {
    pub async fn new(redis_config: application::Redis) -> AppResult<Self> {
        let manager = RedisConnectionManager::new(redis_config.url())
            .map_err(|e| AppError::redis(format!("create manager failed: {e}")))?;

        let pool = Pool::builder()
            .max_size(redis_config.pool_size)
            .build(manager)
            .await
            .map_err(|e| AppError::redis(format!("create pool failed: {e}")))?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> AppResult<Connection<'_>> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::redis(format!("get connection failed: {e}")))
    }

    pub async fn ping(&self) -> AppResult<()> {
        let mut conn = self.get().await?;

        cmd("PING")
            .query_async::<()>(&mut *conn)
            .await
            .map_err(|e| AppError::redis(format!("PING failed: {e}")))
    }

    // --- 封装常用命令 ---

    /// 设置带过期时间的字符串 (SETEX)
    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> AppResult<()> {
        let mut conn = self.get().await?;
        conn.set_ex(key, value, seconds)
            .await
            .map_err(|e| AppError::redis(format!("SETEX failed: {e}")))
    }

    /// 检查 Key 是否存在
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.get().await?;
        conn.exists(key)
            .await
            .map_err(|e| AppError::redis(format!("EXISTS failed: {e}")))
    }

    /// 删除 Key
    pub async fn del(&self, key: &str) -> AppResult<()> {
        let mut conn = self.get().await?;
        conn.del(key)
            .await
            .map_err(|e| AppError::redis(format!("DEL failed: {e}")))
    }

    /// 获取字符串值
    pub async fn get_str(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.get().await?;
        conn.get(key)
            .await
            .map_err(|e| AppError::redis(format!("GET failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use redis::AsyncCommands;

    use super::{RedisClient, application::Redis};

    #[test]
    fn builds_url_without_password() {
        let cfg = Redis {
            host: "192.168.31.218".into(),
            port: 6379,
            password: Some("redis123456".into()),
            db: 0,
            pool_size: 4,
        };

        assert_eq!(cfg.url(), "redis://:redis123456@192.168.31.218:6379/0");
    }

    #[test]
    fn builds_url_with_password_and_db() {
        let cfg = Redis {
            host: "cache.internal".into(),
            port: 6380,
            password: Some("secret".into()),
            db: 2,
            pool_size: 4,
        };

        assert_eq!(cfg.url(), "redis://:secret@cache.internal:6380/2");
    }

    #[tokio::test]
    async fn test_redis_connection_and_ping() {
        let cfg = Redis {
            host: "192.168.31.218".into(),
            port: 6379,
            password: Some("redis123456".into()),
            db: 0,
            pool_size: 2,
        };

        let client = RedisClient::new(cfg)
            .await
            .expect("Failed to create RedisClient pool");

        // 测试 Ping
        let result = client.ping().await;
        assert!(result.is_ok(), "Ping failed: {:?}", result.err());

        let mut conn = client.get().await.expect("Failed to get connection");

        let _: () = conn
            .set("test_key_1", 123)
            .await
            .expect("Set command failed");

        // 使用 &mut *conn 是正确的，因为这会解引用到具体的 Redis 连接
        let _: () = redis::cmd("SET")
            .arg("test_key")
            .arg(123)
            .query_async(&mut *conn)
            .await
            .expect("Set command failed");

        let val: i32 = redis::cmd("GET")
            .arg("test_key")
            .query_async(&mut *conn)
            .await
            .expect("Get command failed");

        assert_eq!(val, 123);
    }
}
