use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// 简化的熔断器管理器
#[derive(Clone)]
pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, Arc<SimpleCircuitBreaker>>>>,
    config: CircuitBreakerConfig,
}

#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
}

/// 简单的熔断器实现
pub struct SimpleCircuitBreaker {
    failure_count: Arc<tokio::sync::RwLock<u32>>,
    last_failure_time: Arc<tokio::sync::RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl SimpleCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            failure_count: Arc::new(tokio::sync::RwLock::new(0)),
            last_failure_time: Arc::new(tokio::sync::RwLock::new(None)),
            config,
        }
    }

    pub async fn is_open(&self) -> bool {
        let last_failure = self.last_failure_time.read().await;
        if let Some(last_time) = *last_failure {
            let failure_count = *self.failure_count.read().await;
            if failure_count >= self.config.failure_threshold {
                // 检查是否超过超时时间
                if last_time.elapsed() < self.config.timeout {
                    return true; // 熔断器仍然打开
                } else {
                    // 超时后尝试半开状态
                    drop(last_failure);
                    *self.failure_count.write().await = 0;
                    *self.last_failure_time.write().await = None;
                }
            }
        }
        false
    }

    pub async fn record_success(&self) {
        *self.failure_count.write().await = 0;
        *self.last_failure_time.write().await = None;
    }

    pub async fn record_failure(&self) {
        let mut count = self.failure_count.write().await;
        *count += 1;
        *self.last_failure_time.write().await = Some(Instant::now());
    }
}

impl CircuitBreakerManager {
    pub fn new(failure_threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config: CircuitBreakerConfig {
                failure_threshold,
                timeout: Duration::from_secs(timeout_seconds),
            },
        }
    }

    /// 获取或创建服务的熔断器
    pub async fn get_or_create(&self, service_name: &str) -> Arc<SimpleCircuitBreaker> {
        let mut breakers = self.breakers.write().await;

        breakers
            .entry(service_name.to_string())
            .or_insert_with(|| Arc::new(SimpleCircuitBreaker::new(self.config.clone())))
            .clone()
    }

    /// 执行带熔断保护的操作
    pub async fn call<F, Fut, T, E>(
        &self,
        service_name: &str,
        f: F,
    ) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let breaker = self.get_or_create(service_name).await;

        // 检查熔断器状态
        if breaker.is_open().await {
            tracing::warn!("Circuit breaker open for service: {}", service_name);
            return Err(CircuitBreakerError::Open);
        }

        // 执行调用
        match f().await {
            Ok(result) => {
                breaker.record_success().await;
                Ok(result)
            }
            Err(e) => {
                breaker.record_failure().await;
                tracing::error!("Service call failed: {}", e);
                Err(CircuitBreakerError::ServiceError(e.to_string()))
            }
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError {
    Open,
    ServiceError(String),
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Circuit breaker is open"),
            Self::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}

impl std::error::Error for CircuitBreakerError {}
