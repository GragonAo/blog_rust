# YAML 配置使用说明

## ✅ 已实现功能

### 1. YAML 配置文件加载
- ✅ 自动从多个路径尝试加载配置文件
- ✅ 加载失败时自动回退到默认配置
- ✅ 支持完整的应用配置结构

### 2. 配置结构

#### application.yaml 配置项：
```yaml
redis:
  host: 192.168.31.218
  port: 6379
  password: redis123456
  pool_size: 5

database:
  url: postgres://postgres:postgres@192.168.31.218:5432/blog_db
  max_connections: 10
  min_connections: 2

snowflake:
  machine_id: 1
  node_id: 1

server:
  bind_addr: 0.0.0.0:5010
```

## 使用方法

### 方式 1：从默认路径加载（推荐）
```rust
let app_config = AppConfig::from_default_yaml()?;
```

会自动尝试以下路径（按顺序）：
1. `src/application.yaml`
2. `modules/user-service/src/application.yaml`
3. `application.yaml`

### 方式 2：从指定路径加载
```rust
let app_config = AppConfig::from_yaml("path/to/config.yaml")?;
```

### 方式 3：使用默认配置（代码硬编码）
```rust
let app_config = AppConfig::default();
```

## 当前实现

在 [main.rs](src/main.rs) 中：
```rust
fn init_app_config() -> Result<AppConfig, AppError> {
    // 优先从 YAML 文件加载，失败则使用默认配置
    AppConfig::from_default_yaml()
        .or_else(|e| {
            eprintln!("Warning: Failed to load config from YAML: {}", e);
            eprintln!("Using default configuration");
            Ok(AppConfig::default())
        })
}
```

这种实现确保：
- ✅ 生产环境可以使用 YAML 配置
- ✅ 开发环境即使没有 YAML 文件也能运行（使用默认配置）
- ✅ 配置错误会有清晰的警告信息

## 测试配置加载

运行测试示例：
```bash
cd /var/local/blog_rust
cargo run -p user-service --example test_config
```

输出示例：
```
=== 测试从 YAML 加载配置 ===

✅ YAML 配置加载成功！

Redis 配置:
  - Host: 192.168.31.218
  - Port: 6379
  - Pool Size: 5

数据库配置:
  - URL: postgres://postgres:postgres@192.168.31.218:5432/blog_db
  - Max Connections: 10
  - Min Connections: 2
...
```

## 修改配置

只需编辑 [src/application.yaml](src/application.yaml) 文件，重启服务即可生效。

## 依赖

已添加 `serde_yml` 依赖到 [Cargo.toml](Cargo.toml)：
```toml
serde_yml.workspace = true
```

## 最佳实践

1. **生产环境**：使用 YAML 配置文件，便于部署时修改
2. **开发环境**：保留 Default 实现作为后备
3. **配置敏感信息**：建议使用环境变量或密钥管理服务
4. **版本控制**：提交 `application.yaml.example`，忽略实际的 `application.yaml`
