use crate::{rpc::account, state::ChampStateMutex};
use pog_proto::rpc::account_server::Account;
pub use pog_proto::rpc::account_server::AccountServer;
use pog_proto::rpc::{BalanceReply, BalanceRequest};

use derive_new::new;
use tonic::{Request, Response, Status};

#[derive(Debug, new)]
pub struct AccountService {
    pub state: ChampStateMutex,
}

#[tonic::async_trait]
impl Account for AccountService {
    async fn get_balance (&self, request: Request<BalanceRequest>) -> Result<Response<BalanceReply>, Status> {
        // We must use .into_inner() as the fields of gRPC requests and responses are private        
        let account_address = request.into_inner().address;

        println!("Got a request for address: {:?}", account_address); 

    
        let state = self
            .state
            .lock()
            .map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        let db_response = state.db.get_latest_block_by_account(&account_address);
        let response = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        let reply = BalanceReply {
            balance: response.block.data.balance,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}