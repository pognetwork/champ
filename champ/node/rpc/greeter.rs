use derive_new::new;
use tonic::{Request, Response, Status};

use crate::ChampStateMutex;

use champ_proto::rpc::greeter_server::Greeter;
pub use champ_proto::rpc::greeter_server::GreeterServer;
use champ_proto::rpc::{HelloReply, HelloRequest};

#[derive(Debug, Default, new)]
pub struct GreeterService {
    pub state: ChampStateMutex,
}

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let state = self.state.lock().unwrap();
        println!("{}", state.username);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}