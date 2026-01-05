# User Service 数据库集成

## 配置

数据库配置在 `application.yaml` 中：

```yaml
database:
  url: postgres://postgres:postgres@192.168.31.218:5432/blog_db
  max_connections: 10
  min_connections: 2
```

## 数据库初始化

1. 确保 PostgreSQL 数据库已启动
2. 创建数据库（如果不存在）：
```bash
psql -U postgres -h 192.168.31.218 -c "CREATE DATABASE blog_db;"
```

3. 运行迁移脚本：
```bash
psql -U postgres -h 192.168.31.218 -d blog_db -f migrations/001_init_users.sql
```

或者使用 sqlx-cli：
```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# 创建数据库
sqlx database create --database-url "postgres://postgres:postgres@192.168.31.218:5432/blog_db"

# 运行迁移
sqlx migrate run --database-url "postgres://postgres:postgres@192.168.31.218:5432/blog_db" --source migrations
```

## 数据库表结构

### users 表
- `id`: BIGINT (主键，使用 Snowflake ID)
- `username`: VARCHAR(255) (唯一)
- `email`: VARCHAR(255) (可选，唯一)
- `password_hash`: VARCHAR(255) (可选)
- `created_at`: TIMESTAMP WITH TIME ZONE
- `updated_at`: TIMESTAMP WITH TIME ZONE

### user_web3_info 表
- `id`: BIGINT (主键，使用 Snowflake ID)
- `user_id`: BIGINT (外键引用 users.id)
- `chain_id`: BIGINT (区块链 ID)
- `web3_address`: VARCHAR(255) (Web3 地址)
- `created_at`: TIMESTAMP WITH TIME ZONE
- `updated_at`: TIMESTAMP WITH TIME ZONE

## 使用示例

在 `user_service.rs` 中使用数据库：

```rust
use crate::domain::model::user::User;

// 查询用户
let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
    .bind(user_id as i64)
    .fetch_one(&self.db_pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to fetch user: {}", e)))?;

// 插入用户
let user_id = self.id_generator.lock().unwrap().real_time_generate();
sqlx::query(
    "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)"
)
    .bind(user_id)
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .execute(&self.db_pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to insert user: {}", e)))?;

// 更新用户
sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
    .bind(&new_username)
    .bind(user_id as i64)
    .execute(&self.db_pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to update user: {}", e)))?;

// 删除用户
sqlx::query("DELETE FROM users WHERE id = $1")
    .bind(user_id as i64)
    .execute(&self.db_pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to delete user: {}", e)))?;
```

## 数据库连接池

数据库连接池在应用启动时初始化，配置参数：
- `max_connections`: 最大连接数
- `min_connections`: 最小连接数

连接池通过 `PgPool` 在整个应用中共享，自动处理连接的创建、复用和回收。
