use crate::state::ChampStateArc;
use pog_jwt::*;
use pog_proto::rpc::node_user::*;

pub use pog_proto::rpc::node_user::node_user_server::{NodeUser, NodeUserServer};
use tonic::{Response, Status};
use tracing::debug;

#[derive(Debug)]
pub struct NodeUserService {
    pub state: ChampStateArc,
}

impl NodeUserService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

#[tonic::async_trait]
impl NodeUser for NodeUserService {
    async fn login(
        &self,
        request: tonic::Request<LoginRequest>,
    ) -> Result<tonic::Response<LoginReply>, tonic::Status> {
        debug!("user logging in");

        let req = request.into_inner();
        let username = req.username.to_lowercase();
        let password = req.password;

        let (jwt_key, user) = {
            let config = &self.state.config.read().await;
            let user = config.node_users.get(&username);
            let jwt_key = config.admin.jwt_private_key.clone();
            let user = user.ok_or_else(|| Status::new(tonic::Code::Internal, "invalid username"))?;
            (jwt_key, user.clone())
        };

        crypto::password::verify(password.as_bytes(), &user.password_hash)
            .map_err(|_| Status::new(tonic::Code::Internal, "invalid password"))?;

        let expires_in = 10000;
        let token = create(&user.user_id, &username, expires_in, jwt_key.as_bytes())
            .map_err(|_| Status::new(tonic::Code::Internal, "could not create token"))?;
        Ok(Response::new(LoginReply {
            token,
        }))
    }
}
