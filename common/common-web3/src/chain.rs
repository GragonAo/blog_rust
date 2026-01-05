use common_core::AppError;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    Evm = 1,       // Ethereum Mainnet
    Bsc = 56,      // Binance Smart Chain
    Polygon = 137, // Polygon
    Solana = 101,  // 假设给 Solana 定义一个内部标识码
}

impl Chain {
    /// 获取链的友好名称
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Evm => "Ethereum",
            Chain::Bsc => "BSC",
            Chain::Polygon => "Polygon",
            Chain::Solana => "Solana",
        }
    }
}

/// 实现从数字 ID 到枚举的转换
impl TryFrom<u64> for Chain {
    type Error = AppError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Chain::Evm),
            56 => Ok(Chain::Bsc),
            137 => Ok(Chain::Polygon),
            101 => Ok(Chain::Solana),
            _ => Err(AppError::internal(format!(
                "Unsupported Chain ID: {}",
                value
            ))),
        }
    }
}
