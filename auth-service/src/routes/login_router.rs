use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use common_web3::chain::Chain;

use crate::{
    AppServiceState,
    domain::{request::login::LoginWeb3Request, response::login::LoginResponse},
};
use common_web::domain::r::R;

pub fn router() -> Router<AppServiceState> {
    Router::new()
        .route("/login/web3", post(login_web3_wallet))
        .route("/login/nonce/{chain_id}", get(get_login_web3_nonce))
}

async fn get_login_web3_nonce(
    Path(chain_id): Path<u64>,
    State(state): State<AppServiceState>,
) -> Json<R<String>> {
    if let Err(e) = Chain::try_from(chain_id) {
        return Json(R::error(e.to_string(), StatusCode::BAD_REQUEST.as_u16()));
    }
    match state.login_service.get_login_web3_nonce(chain_id).await {
        Ok(nonce) => Json(R::ok(nonce)),
        Err(err) => Json(R::error(
            err.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        )),
    }
}

async fn login_web3_wallet(
    State(state): State<AppServiceState>,
    Json(body): Json<LoginWeb3Request>,
) -> Json<R<LoginResponse>> {
    match state
        .login_service
        .login_web3_wallet(body.signature, body.message)
        .await
    {
        Ok(recovered_addr) => {
            if recovered_addr == body.address {
                //TODO 生成JWT
                Json(R::ok(LoginResponse {
                    access_token: "header.payload.signature".to_string(),
                    expire_in: 3600,
                    refresh_token: "refresh_token_here".to_string(),
                    refresh_expire_in: 86400,
                    client_id: "test".to_string(),
                }))
            } else {
                Json(R::error(
                    "Address mismatch",
                    StatusCode::UNAUTHORIZED.as_u16(),
                ))
            }
        }
        Err(err) => Json(R::error(
            err.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        )),
    }
}
