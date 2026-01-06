use crate::domain::model::user::User;
use async_trait::async_trait;
use common_core::AppError;
use sqlx::{PgConnection, PgPool};

/// 用户数据访问层 trait
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        id: i64,
    ) -> Result<Option<User>, AppError>;
    async fn find_by_username(
        &self,
        executor: &mut PgConnection,
        username: &str,
    ) -> Result<Option<User>, AppError>;
    async fn find_by_email(
        &self,
        executor: &mut PgConnection,
        email: &str,
    ) -> Result<Option<User>, AppError>;
    async fn inster(&self, executor: &mut PgConnection, user: &User) -> Result<(), AppError>;
    async fn update(&self, executor: &mut PgConnection, user: &User) -> Result<(), AppError>;
    async fn delete(&self, executor: &mut PgConnection, id: i64) -> Result<(), AppError>;
}

pub struct UserRepositoryImpl {
    // 虽然保留了 pool，但在事务模式下，方法内部应优先使用传入的 executor
    pub db_pool: PgPool,
}

impl UserRepositoryImpl {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(
        &self,
        executor: &mut PgConnection,
        id: i64,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(executor) // 关键：使用传入的 executor
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by id: {}", e)))
    }

    async fn find_by_username(
        &self,
        executor: &mut PgConnection,
        username: &str,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by username: {}", e)))
    }

    async fn find_by_email(
        &self,
        executor: &mut PgConnection,
        email: &str,
    ) -> Result<Option<User>, AppError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to fetch user by email: {}", e)))
    }

    async fn inster(&self, executor: &mut PgConnection, user: &User) -> Result<(), AppError> {
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
        .execute(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to create user: {}", e)))?;
        Ok(())
    }

    async fn update(&self, executor: &mut PgConnection, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET username = $1, email = $2, password_hash = $3, updated_at = NOW() 
             WHERE id = $4",
        )
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.id)
        .execute(executor)
        .await
        .map_err(|e| AppError::Db(format!("Failed to update user: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, executor: &mut PgConnection, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Db(format!("Failed to delete user: {}", e)))?;
        Ok(())
    }
}
