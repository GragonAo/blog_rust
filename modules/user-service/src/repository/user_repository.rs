use crate::domain::model::user::User;
use async_trait::async_trait;
use common_core::AppError;
use sqlx::PgPool;

/// 用户数据访问层 trait
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// 根据 ID 查询用户
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError>;

    /// 根据用户名查询用户
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;

    /// 根据邮箱查询用户
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    /// 创建用户
    async fn create(&self, user: &User) -> Result<(), AppError>;

    /// 更新用户
    async fn update(&self, user: &User) -> Result<(), AppError>;

    /// 删除用户
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}

/// 用户数据访问层实现
pub struct UserRepositoryImpl {
    db_pool: PgPool,
}

impl UserRepositoryImpl {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by id: {}", e)))?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by username: {}", e)))?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by email: {}", e)))?;

        Ok(user)
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Db(format!("Failed to create user: {}", e)))?;

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET username = $1, email = $2, password_hash = $3, updated_at = NOW() 
             WHERE id = $4",
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Db(format!("Failed to update user: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| AppError::Db(format!("Failed to delete user: {}", e)))?;

        Ok(())
    }
}
