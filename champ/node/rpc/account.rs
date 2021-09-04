use crate::state::ChampStateMutex;
use pog_proto::rpc::account_server::Account;
pub use pog_proto::rpc::account_server::AccountServer;
use pog_proto::rpc::{BalanceReply, BalanceRequest};

use derive_new::new;
use tonic::{Request, Response, Status};

#[derive(Debug, Default, new)]
pub struct AccountService {
    pub state: ChampStateMutex,
}

#[tonic::async_trait]
impl Account for AccountService {
    async fn get_balance (
        &self,
        request: Request<BalanceRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<BalanceReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request for address: {:?}", request.into_inner().address); // We must use .into_inner() as the fields of gRPC requests and responses are private

        // TODO: check and get storage for account address
        
        let _state = self
            .state
            .lock()
            .map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        let reply = BalanceReply {
            balance: 0, // DB response here
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}
