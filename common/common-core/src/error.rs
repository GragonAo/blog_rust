use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Redis Error: {0}")]
    Redis(String),

    #[error("Database Error: {0}")]
    Db(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn redis(msg: impl Into<String>) -> Self {
        Self::Redis(msg.into())
    }

    pub fn db(msg: impl Into<String>) -> Self {
        Self::Db(msg.into())
    }
}
