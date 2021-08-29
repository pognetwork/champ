use crate::state::ChampStateMutex;
use champ_proto::rpc::greeter_server::Greeter;
pub use champ_proto::rpc::greeter_server::GreeterServer;
use champ_proto::rpc::{HelloReply, HelloRequest};

use derive_new::new;
use tonic::{Request, Response, Status};

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

        let state = self
            .state
            .lock()
            .map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        println!("{}", state.username);

        let req = request.into_inner(); // We must use .into_inner() as the fields of gRPC requests and responses are private
        let reply = HelloReply {
            message: format!("Hello {}!", req.name).into(),
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
