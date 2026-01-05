use bb8_redis::{
    RedisConnectionManager,
    bb8::{Pool, PooledConnection},
};
use redis::{RedisError, cmd};

pub mod application;

pub type ConnectionPool = Pool<RedisConnectionManager>;
pub type Connection<'a> = PooledConnection<'a, RedisConnectionManager>;

#[derive(Clone)]
pub struct RedisClient {
    pool: ConnectionPool,
}

impl RedisClient {
    pub async fn new(
        redis_config: application::Redis,
    ) -> Result<Self, bb8_redis::bb8::RunError<RedisError>> {
        let manager = RedisConnectionManager::new(redis_config.url())?;

        let pool = Pool::builder()
            .max_size(redis_config.pool_size)
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> Result<Connection<'_>, bb8_redis::bb8::RunError<RedisError>> {
        self.pool.get().await
    }

    pub async fn ping(&self) -> Result<(), bb8_redis::bb8::RunError<RedisError>> {
        let mut conn = self.get().await?;

        cmd("PING")
            .query_async::<()>(&mut *conn)
            .await
            .map_err(bb8_redis::bb8::RunError::User)
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
