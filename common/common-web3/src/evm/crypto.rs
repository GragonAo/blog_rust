use common_core::AppError;
use web3::signing::{keccak256, recover};

pub struct EvmCrypto;

impl EvmCrypto {
    /// 核心能力：通过消息和签名恢复出 0x 地址
    pub fn recover_address(message: &str, signature: &str) -> Result<String, AppError> {
        let sig_hex = signature.strip_prefix("0x").unwrap_or(signature);
        let sig_bytes =
            hex::decode(sig_hex).map_err(|_| AppError::internal("Invalid hex signature"))?;

        if sig_bytes.len() != 65 {
            return Err(AppError::internal("Signature must be 65 bytes"));
        }

        // 构造以太坊特定格式消息
        let eth_msg = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let msg_hash = keccak256(eth_msg.as_bytes());

        // 恢复 ID (v) 处理
        let v = sig_bytes[64];
        let recovery_id = if v >= 27 { v - 27 } else { v } as i32;

        let addr = recover(&msg_hash, &sig_bytes[..64], recovery_id)
            .map_err(|e| AppError::internal(format!("Recovery failed: {}", e)))?;

        Ok(format!("{:?}", addr).to_lowercase())
    }
}
