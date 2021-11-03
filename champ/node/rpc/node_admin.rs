use crate::auth::permissions::verify_perms;
use pog_proto::rpc::node_admin::*;
use tonic::{Response, Status};

use crate::state::ChampStateArc;

pub use pog_proto::rpc::node_admin::node_admin_server::{NodeAdmin, NodeAdminServer};

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
        _request: tonic::Request<GetPeersRequest>,
    ) -> Result<tonic::Response<GetPeersResponse>, tonic::Status> {
        unimplemented!()
    }
    async fn get_version(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetVersionResponse>, tonic::Status> {
        verify_perms(&request, "admin.read")?;
        Ok(Response::new(GetVersionResponse {
            version: VERSION.to_string(),
        }))
    }
    async fn upgrade_node(
        &self,
        _request: tonic::Request<UpgradeNodeRequest>,
    ) -> Result<tonic::Response<UpgradeNodeResponse>, tonic::Status> {
        unimplemented!()
    }
    async fn get_pending_blocks(
        &self,
        _request: tonic::Request<GetPendingBlocksRequest>,
    ) -> Result<tonic::Response<GetPendingBlocksReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_block_pool_size(
        &self,
        request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetBlockPoolSizeReply>, tonic::Status> {
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
        _request: tonic::Request<GetNodeStatusRequest>,
    ) -> Result<tonic::Response<GetNodeStatusReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_mode(
        &self,
        _request: tonic::Request<GetModeRequest>,
    ) -> Result<tonic::Response<GetModeReply>, tonic::Status> {
        unimplemented!()
    }
    async fn set_mode(
        &self,
        _request: tonic::Request<SetModeRequest>,
    ) -> Result<tonic::Response<SetModeReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_node_name(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetNodeNameReply>, tonic::Status> {
        let name = &self.state.config.read().await.admin.node_name;
        Ok(Response::new(GetNodeNameReply {
            name: name.to_string(),
        }))
    }
    async fn set_node_name(
        &self,
        request: tonic::Request<SetNodeNameRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        verify_perms(&request, "admin.write")?;
        let new_name = request.into_inner().new_name;
        let mut config = self.state.config.write().await;
        config.admin.node_name = new_name;
        config.write().map_err(|_| Status::new(tonic::Code::Internal, "could not update the node name"))?;
        Ok(Response::new(Empty {}))
    }
    async fn get_chain(
        &self,
        _request: tonic::Request<GetChainRequest>,
    ) -> Result<tonic::Response<GetChainReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_logs(
        &self,
        _request: tonic::Request<GetLogsRequest>,
    ) -> Result<tonic::Response<GetLogsReply>, tonic::Status> {
        unimplemented!()
    }
}
