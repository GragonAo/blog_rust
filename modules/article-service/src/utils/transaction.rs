/// 事务宏，自动处理事务的开始、提交和回滚
#[macro_export]
macro_rules! with_transaction {
    ($pool:expr, |$conn:ident| async $body:block) => {
        {
            let mut tx = $pool.begin().await.map_err(|e| common_core::AppError::db(e.to_string()))?;
            let $conn = &mut *tx;
            
            let result: Result<_, common_core::AppError> = async $body.await;
            
            match result {
                Ok(value) => {
                    tx.commit().await.map_err(|e| common_core::AppError::db(e.to_string()))?;
                    Ok(value)
                }
                Err(e) => {
                    let _ = tx.rollback().await;
                    Err(e)
                }
            }
        }
    };
}