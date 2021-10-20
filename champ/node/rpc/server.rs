use std::{net::SocketAddr, time::Duration};

use crate::{
    rpc::account::{AccountServer, AccountService},
    state::ChampStateArc,
};
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
        let account_server = AccountServer::new(AccountService::new(self.state.clone()));
        println!("starting rpc server at {}", addr);

        // The stack of middleware that our service will be wrapped in
        let layer = tower::ServiceBuilder::new().timeout(Duration::from_secs(30)).into_inner();

        Server::builder()
            .accept_http1(true)
            .layer(layer)
            .add_service(tonic_web::enable(account_server))
            .serve(addr)
            .await?;

        Ok(())
    }
}
