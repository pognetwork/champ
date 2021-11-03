use crate::interceptors::interceptor_auth;
use crate::rpc::block::{BlockServer, BlockService};
use crate::rpc::node_admin::{NodeAdminServer, NodeAdminService};
use crate::rpc::node_user::{NodeUserServer, NodeUserService};
use crate::rpc::node_wallet_manager::{NodeWalletManagerServer, NodeWalletManagerService};
use crate::state::ChampStateArc;
use std::{net::SocketAddr, time::Duration};

use tonic::transport::Server;
use tonic::Request;

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
        let (public_key, users) = {
            let cfg = &self.state.config.read().await;
            (cfg.admin.jwt_public_key.to_owned(), cfg.node_users.clone())
        };
        let cloned_public_key = public_key.clone();
        let cloned_users = users.clone();

        let block_server = BlockServer::new(BlockService::new(self.state.clone()));
        let node_admin_server = NodeAdminServer::with_interceptor(
            NodeAdminService::new(self.state.clone()),
            move |request: Request<()>| interceptor_auth(request, &public_key, &users),
        );
        let node_wallet_manager_server = NodeWalletManagerServer::with_interceptor(
            NodeWalletManagerService::new(self.state.clone()),
            move |request| interceptor_auth(request, &cloned_public_key, &cloned_users),
        );
        let node_user = NodeUserServer::new(NodeUserService::new(self.state.clone()));

        println!("starting rpc server at {}", addr);

        // The stack of middleware that our service will be wrapped in
        let layer = tower::ServiceBuilder::new().timeout(Duration::from_secs(30)).into_inner();

        Server::builder()
            .accept_http1(true)
            .layer(layer)
            .add_service(tonic_web::enable(block_server))
            .add_service(tonic_web::enable(node_admin_server))
            .add_service(tonic_web::enable(node_wallet_manager_server))
            .add_service(tonic_web::enable(node_user))
            .serve(addr)
            .await?;
        Ok(())
    }
}
