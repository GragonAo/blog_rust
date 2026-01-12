use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub struct TracingService;
pub mod application;

impl TracingService {
    pub fn init(log_config: &application::Logs) -> WorkerGuard {
        let path = Path::new(&log_config.path);
        // 1. 获取父目录 (Default 为当前目录 ".")
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        // 2. 获取文件名前缀 (去除扩展名)
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
        // 3. 初始化 file_appender
        let file_appender = tracing_appender::rolling::daily(parent, file_stem);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let stdout = std::io::stdout.and(non_blocking);
        tracing_subscriber::fmt()
            .with_target(false)
            .with_writer(stdout)
            .compact()
            .init();

        // 设置 panic hook，将 panic 信息记录到 user-service 日志中
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // 1. 记录 panic 信息到 tracing 系统
            let backtrace = std::backtrace::Backtrace::capture();
            if let Some(location) = panic_info.location() {
                tracing::error!(
                    message = %panic_info,
                    panic.file = location.file(),
                    panic.line = location.line(),
                    panic.backtrace = ?backtrace,
                    "Service panicked"
                );
            } else {
                tracing::error!(
                    message = %panic_info,
                    panic.backtrace = ?backtrace,
                    "Service panicked"
                );
            }

            // 2. 调用之前的 hook（通常会打印到 stderr）
            prev_hook(panic_info);
        }));

        guard
    }
}
