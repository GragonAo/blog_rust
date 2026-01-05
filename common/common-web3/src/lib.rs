pub mod chain;
pub mod evm;
pub mod solana;
use common_core::AppError;
pub use evm::crypto::EvmCrypto;

use crate::chain::Chain;

pub struct Web3Recover;

impl Web3Recover {
    /// 原子能力：签名 + 消息 -> 地址
    pub fn get_address(chain: Chain, message: &str, signature: &str) -> Result<String, AppError> {
        match chain {
            Chain::Evm | Chain::Bsc | Chain::Polygon => {
                EvmCrypto::recover_address(message, signature)
            }
            Chain::Solana => {
                // TODO: 实现 Solana 的地址恢复 (Ed25519 不需要 recovery_id，通常直接传公钥校验)
                // 或者在一些链上，签名本身就包含了公钥信息
                Err(AppError::internal("Solana not yet implemented"))
            }
        }
    }
}
