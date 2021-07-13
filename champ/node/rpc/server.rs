use crate::rpc::greeter::{GreeterServer, GreeterService};
use tonic::transport::Server;

pub struct RpcServer {}

impl RpcServer {
    pub async fn start(addr: String) -> Result<(), Box<dyn std::error::Error>> {
        let greeter = GreeterService::default();

        let ad = addr.parse().unwrap();

        println!("starting rpc server");

        let server = Server::builder()
            .add_service(GreeterServer::new(greeter))
            .serve(ad);
        server.await?;

        Ok(())
    }
}
