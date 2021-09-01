use derive_new::new;
use std::{net::SocketAddr, time::Duration};

use crate::{
    rpc::greeter::{GreeterServer, GreeterService},
    state::ChampStateMutex,
};
use tonic::transport::Server;

#[derive(Debug, new)]
pub struct RpcServer {
    state: ChampStateMutex,
}

impl RpcServer {
    pub async fn start(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let greeter_server = GreeterServer::new(GreeterService::new(self.state.clone()));
        println!("starting rpc server at {}", addr);

        // The stack of middleware that our service will be wrapped in
        let layer = tower::ServiceBuilder::new()
            .timeout(Duration::from_secs(30))
            .into_inner();

        Server::builder()
            .accept_http1(true)
            .layer(layer)
            .add_service(tonic_web::enable(greeter_server))
            .serve(addr)
            .await?;

        Ok(())
    }
}
