use crate::auth::permissions::verify_perms;
use crate::state::ChampStateArc;
use pog_proto::{api::Empty, rpc::node_admin::*};
use tonic::{Response, Status};

pub use pog_proto::rpc::node_admin::node_admin_server::{NodeAdmin, NodeAdminServer};
use tracing::debug;

#[derive(Debug)]
pub struct NodeAdminService {
    pub state: ChampStateArc,
}

impl NodeAdminService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tonic::async_trait]
impl NodeAdmin for NodeAdminService {
    async fn get_peers(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetPeersResponse>, tonic::Status> {
        unimplemented!()
    }

    async fn get_version(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetVersionResponse>, tonic::Status> {
        verify_perms(&request, "admin.read")?;
        debug!("getting node version");

        Ok(Response::new(GetVersionResponse {
            version: VERSION.to_string(),
        }))
    }

    async fn upgrade_node(
        &self,
        _request: tonic::Request<UpgradeNodeRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        unimplemented!()
    }

    async fn get_pending_blocks(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetPendingBlocksReply>, tonic::Status> {
        unimplemented!()
    }

    async fn get_block_pool_size(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetBlockPoolSizeReply>, tonic::Status> {
        debug!("getting block pool size");

        verify_perms(&request, "admin.read")?;
        Ok(Response::new(GetBlockPoolSizeReply {
            length: self
                .state
                .blockpool_client
                .get_queue_size()
                .await
                .map_err(|_| Status::new(tonic::Code::Internal, "could not get queue length"))?,
        }))
    }
    async fn get_node_status(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetNodeStatusReply>, tonic::Status> {
        unimplemented!()
    }

    async fn get_mode(&self, request: tonic::Request<Empty>) -> Result<tonic::Response<GetModeReply>, tonic::Status> {
        debug!("getting node mode");

        verify_perms(&request, "admin.read")?;
        let mode = &self.state.config.read().await.consensus.mode;
        Ok(Response::new(GetModeReply {
            current_mode: *mode as i32,
        }))
    }

    async fn set_mode(
        &self,
        request: tonic::Request<SetModeRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        debug!("setting node mode");

        verify_perms(&request, "admin.write")?;
        let new_mode = Mode::from_i32(request.into_inner().mode)
            .ok_or_else(|| Status::new(tonic::Code::Internal, "invalid mode"))?;
        let mut config = self.state.config.write().await;
        config.consensus.mode = new_mode;
        config.write().map_err(|_| Status::new(tonic::Code::Internal, "could not update the node name"))?;
        Ok(Response::new(Empty {}))
    }

    async fn get_node_name(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetNodeNameReply>, tonic::Status> {
        debug!("getting node name");

        verify_perms(&request, "admin.read")?;
        let name = &self.state.config.read().await.admin.node_name;
        Ok(Response::new(GetNodeNameReply {
            name: name.to_string(),
        }))
    }
    async fn set_node_name(
        &self,
        request: tonic::Request<SetNodeNameRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        debug!("setting node name");

        verify_perms(&request, "admin.write")?;
        let new_name = request.into_inner().new_name;
        //TODO: Add length checks
        let mut config = self.state.config.write().await;
        config.admin.node_name = new_name;
        config.write().map_err(|_| Status::new(tonic::Code::Internal, "could not update the node name"))?;
        Ok(Response::new(Empty {}))
    }

    async fn get_chain(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetChainReply>, tonic::Status> {
        debug!("getting node chain");

        verify_perms(&request, "admin.read")?;
        let config = self.state.config.read().await;
        Ok(Response::new(GetChainReply {
            current_chain: config.consensus.chain.clone(),
        }))
    }

    async fn get_logs(
        &self,
        request: tonic::Request<GetLogsRequest>,
    ) -> Result<tonic::Response<GetLogsReply>, tonic::Status> {
        debug!("getting node logs (this)");

        verify_perms(&request, "admin.logs")?;
        unimplemented!()
    }
}
