use crate::state::ChampStateArc;
use std::{net::SocketAddr, time::Duration};

use pog_proto::rpc::{
    node_admin::node_admin_server::NodeAdminServer,
    node_wallet_manager::node_wallet_manager_server::NodeWalletManagerServer,
};

use crate::rpc::block::{BlockServer, BlockService};
use crate::rpc::node_admin::NodeAdminService;
use crate::rpc::node_wallet_manager::NodeWalletManagerService;

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
        let account_server = BlockServer::new(BlockService::new(self.state.clone()));
        let admin_server = NodeAdminServer::new(NodeAdminService::new(self.state.clone()));
        let private_server = NodeWalletManagerServer::new(NodeWalletManagerService::new(self.state.clone()));
        println!("starting rpc server at {}", addr);

        // The stack of middleware that our service will be wrapped in
        let layer = tower::ServiceBuilder::new().timeout(Duration::from_secs(30)).into_inner();

        Server::builder()
            .accept_http1(true)
            .layer(layer)
            .add_service(tonic_web::enable(account_server))
            .add_service(tonic_web::enable(admin_server))
            .add_service(tonic_web::enable(private_server))
            .serve(addr)
            .await?;
        Ok(())
    }
}
