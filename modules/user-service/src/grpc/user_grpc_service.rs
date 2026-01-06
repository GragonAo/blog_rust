use common_proto::user::{
    RegisterType, RegisterUserReq, RegisterUserRes, UserInfoReq, UserInfoRes, Web3InfoReq,
    user_service_server::{UserService as UserServiceTrait, UserServiceServer},
};
use tonic::{Request, Response, Status};

use crate::{
    domain::bo::user_bo::{UserBo, UserInfoBo, Web3UserInfoBo},
    startup::AppState,
};

/// gRPC 服务实现
pub struct UserGrpcService {
    app_state: AppState,
}

impl UserGrpcService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub fn into_server(self) -> UserServiceServer<Self> {
        UserServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl UserServiceTrait for UserGrpcService {
    async fn get_user_info(
        &self,
        request: Request<UserInfoReq>,
    ) -> Result<Response<UserInfoRes>, Status> {
        let req = request.into_inner();

        let user_info = self
            .app_state
            .user_service
            .get_user_info(req.user_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get user info: {}", e)))?;

        match user_info {
            Some(user_info) => {
                let response = UserInfoRes {
                    id: user_info.user.id,
                    username: user_info.user.username,
                    email: user_info.user.email,
                    created_at: user_info.user.created_at.to_rfc3339(),
                    updated_at: user_info.user.updated_at.to_rfc3339(),
                    web3_info: user_info.web3_info.map(|w| common_proto::user::Web3Info {
                        chain_id: w.chain_id,
                        address: w.address,
                    }),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }

    async fn get_user_info_by_web3(
        &self,
        request: Request<Web3InfoReq>,
    ) -> Result<Response<UserInfoRes>, Status> {
        let req = request.into_inner();

        let user_info = self
            .app_state
            .user_service
            .get_user_info_by_web3(req.chain_id, req.address)
            .await
            .map_err(|e| Status::internal(format!("Failed to get user info by web3: {}", e)))?;

        match user_info {
            Some(user_info) => {
                let response = UserInfoRes {
                    id: user_info.user.id,
                    username: user_info.user.username,
                    email: user_info.user.email,
                    created_at: user_info.user.created_at.to_rfc3339(),
                    updated_at: user_info.user.updated_at.to_rfc3339(),
                    web3_info: user_info.web3_info.map(|w| common_proto::user::Web3Info {
                        chain_id: w.chain_id,
                        address: w.address,
                    }),
                };
                Ok(Response::new(response))
            }
            None => Err(Status::not_found("User not found")),
        }
    }

    async fn register_user(
        &self,
        request: Request<RegisterUserReq>,
    ) -> Result<Response<RegisterUserRes>, Status> {
        let req = request.into_inner();

        let register_type =
            RegisterType::try_from(req.register_type).unwrap_or(RegisterType::Unspecified);

        let normalize_opt_string = |value: Option<String>| {
            value.and_then(|s| {
                let s = s.trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            })
        };

        let username_opt = normalize_opt_string(req.username);
        let email_opt = normalize_opt_string(req.email);
        let password_opt = normalize_opt_string(req.password);
        let web3_address_opt = normalize_opt_string(req.web3_address);
        let web3_chain_id_opt = req.web3_chain_id;

        let (username, web3_info) = match register_type {
            RegisterType::Web3 => {
                let web3_address = web3_address_opt
                    .ok_or_else(|| Status::invalid_argument("web3_address is required"))?;
                let chain_id = web3_chain_id_opt
                    .ok_or_else(|| Status::invalid_argument("web3_chain_id is required"))?;

                let username = username_opt.unwrap_or_else(|| web3_address.clone());
                let web3_info = Some(Web3UserInfoBo {
                    chain_id,
                    address: web3_address,
                });

                (username, web3_info)
            }

            RegisterType::Email => {
                let email = email_opt
                    .clone()
                    .ok_or_else(|| Status::invalid_argument("email is required"))?;
                let username = username_opt.unwrap_or(email);
                (username, None)
            }

            RegisterType::Username => {
                let _username =
                    username_opt.ok_or_else(|| Status::invalid_argument("username is required"))?;
                let _password =
                    password_opt.ok_or_else(|| Status::invalid_argument("password is required"))?;
                return Err(Status::unimplemented(
                    "username/password registration not implemented yet",
                ));
            }

            RegisterType::Unspecified => {
                return Err(Status::invalid_argument("register_type is required"));
            }
        };

        let user_info_bo = UserInfoBo {
            user: UserBo {
                id: None,
                username,
                email: email_opt,
                password_hash: None,
            },
            web3_info,
        };

        // 3. 调用 Service
        let user_id = self
            .app_state
            .user_service
            .create_user(user_info_bo)
            .await
            .map_err(|e| Status::internal(format!("Failed to create user: {}", e)))?;

        Ok(Response::new(RegisterUserRes { user_id }))
    }
}
