use crate::state::ChampStateArc;
use std::{net::SocketAddr, time::Duration};

use crate::rpc::block::{BlockServer, BlockService};
use crate::rpc::node_admin::{NodeAdminServer, NodeAdminService};
use crate::rpc::node_user::{NodeUserServer, NodeUserService};
use crate::rpc::node_wallet_manager::{NodeWalletManagerServer, NodeWalletManagerService};

use tonic::transport::Server;

#[derive(Debug)]
pub struct RpcServer {
    state: ChampStateArc,
}

impl RpcServer {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }

    pub async fn start(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let wallet_server = BlockServer::new(BlockService::new(self.state.clone()));
        let node_admin_server = NodeAdminServer::new(NodeAdminService::new(self.state.clone()));
        let node_wallet_manager_server =
            NodeWalletManagerServer::new(NodeWalletManagerService::new(self.state.clone()));
        let node_user = NodeUserServer::new(NodeUserService::new(self.state.clone()));
        println!("starting rpc server at {}", addr);

        // The stack of middleware that our service will be wrapped in
        let layer = tower::ServiceBuilder::new().timeout(Duration::from_secs(30)).into_inner();

        Server::builder()
            .accept_http1(true)
            .layer(layer)
            .add_service(tonic_web::enable(wallet_server))
            .add_service(tonic_web::enable(node_admin_server))
            .add_service(tonic_web::enable(node_wallet_manager_server))
            .add_service(tonic_web::enable(node_user))
            .serve(addr)
            .await?;
        Ok(())
    }
}
