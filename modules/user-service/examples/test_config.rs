// 测试配置加载
use user_service::config::application::AppConfig;

fn main() {
    println!("=== 测试从 YAML 加载配置 ===\n");

    match AppConfig::from_default_yaml() {
        Ok(config) => {
            println!("✅ YAML 配置加载成功！\n");
            println!("Redis 配置:");
            println!("  - Host: {}", config.redis.host);
            println!("  - Port: {}", config.redis.port);
            println!("  - Pool Size: {}", config.redis.pool_size);

            println!("\n数据库配置:");
            println!("  - URL: {}", config.database.url);
            println!("  - Max Connections: {}", config.database.max_connections);
            println!("  - Min Connections: {}", config.database.min_connections);

            println!("\nSnowflake 配置:");
            println!("  - Machine ID: {}", config.snowflake.machine_id);
            println!("  - Node ID: {}", config.snowflake.node_id);

            println!("\n服务器配置:");
            println!("  - Bind Address: {}", config.server.bind_addr);
        }
        Err(e) => {
            println!("❌ YAML 配置加载失败: {}", e);
            println!("\n使用默认配置:");
            let config = AppConfig::default();
            println!("  - Redis: {}:{}", config.redis.host, config.redis.port);
            println!("  - Server: {}", config.server.bind_addr);
        }
    }
}
