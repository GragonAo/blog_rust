use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,   // User ID (Subject)
    pub exp: usize, // Expiration time (timestamp)
    pub iat: usize, // Issued at (timestamp)
}

pub struct JwtUtils;

impl JwtUtils {
    /// 生成 Access Token
    pub fn create_token(secret: String, user_id: i64, hours: u64) -> Result<String, AppError> {
        let exp = Utc::now()
            .checked_add_signed(Duration::hours(hours as i64))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: exp as usize,
            iat: Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("JWT encode error: {}", e)))
    }

    /// 验证并解析 Token
    pub fn verify_token(secret: String, token: String) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Internal("Invalid or expired token".into()))
    }
}
