use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use common_core::{AppError, utils::jwt_utils::JwtUtils};
use common_web::domain::r::R;
use common_web3::chain::Chain;

use crate::error::ApiError;

use crate::{
    AppState,
    domain::{request::login::LoginWeb3Request, response::login::LoginResponse},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/web3-login", post(login_web3_wallet))
        .route("/web3-login/nonce/{chain_id}", get(get_login_web3_nonce))
}

async fn get_login_web3_nonce(
    Path(chain_id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<R<String>>, ApiError> {
    Chain::try_from(chain_id)
        .map_err(|e| AppError::Internal(format!("Invalid chain id: {}", e)))?;

    let nonce = state
        .login_service
        .get_login_web3_nonce(chain_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(R::ok(nonce)))
}

async fn login_web3_wallet(
    State(state): State<AppState>,
    Json(body): Json<LoginWeb3Request>,
) -> Result<Json<R<LoginResponse>>, ApiError> {
    let recovered_addr = state
        .login_service
        .login_web3_wallet(body.signature, body.message)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if recovered_addr != body.address {
        return Ok(Json(R::error(
            "Address mismatch",
            StatusCode::UNAUTHORIZED.as_u16(),
        )));
    }

    let user_id = 123;
    let jwt_config = &state.app_config.jwt;

    let access_token = JwtUtils::create_token(
        jwt_config.secret.clone(),
        user_id,
        jwt_config.expiration_hours,
    )?;

    let refresh_token = JwtUtils::create_token(
        jwt_config.secret.clone(),
        user_id,
        jwt_config.refresh_expiration_hours,
    )?;

    Ok(Json(R::ok(LoginResponse {
        access_token,
        expire_in: jwt_config.expiration_hours * 3600,
        refresh_token,
        refresh_expire_in: jwt_config.refresh_expiration_hours * 3600,
        client_id: "test".to_string(),
    })))
}
