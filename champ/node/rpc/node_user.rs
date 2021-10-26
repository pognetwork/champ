use crate::state::ChampStateArc;
use pog_jwt::*;
use pog_proto::rpc::node_user::*;

pub use pog_proto::rpc::node_user::node_user_server::{NodeUser, NodeUserServer};
use tonic::{Response, Status};

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
    async fn login(&self, request: tonic::Request<LoginRequest>) -> Result<tonic::Response<LoginReply>, tonic::Status> {
        let username = request.into_inner().username;
        let private_key = &self.state.config.read().await.admin.jwt_private_key;
        let expires_in = 10000;
        let token = create(&username, expires_in, private_key.as_bytes())
            .map_err(|_| Status::new(tonic::Code::Internal, "could not create token"))?;
        Ok(Response::new(LoginReply {
            token: token,
        }))
    }
}
