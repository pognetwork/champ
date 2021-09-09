use crate::{state::ChampStateMutex};
use pog_proto::rpc::account_server::Account;
pub use pog_proto::rpc::account_server::AccountServer;
use pog_proto::rpc::{BalanceReply, BalanceRequest};
use pog_proto::rpc::{NextBlockHeightRequest, NextBlockHeightReply};

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

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(&account_address).await;
        let response = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        match &response.data {
            Some(data) => Ok(Response::new(BalanceReply {balance: data.balance})),
            None => Err(Status::new(tonic::Code::Internal, "missing Block data")),
        }
    }
    async fn get_next_block_height (&self, request: Request<NextBlockHeightRequest>) -> Result<Response<NextBlockHeightReply>, Status> {
        let account_address = request.into_inner().address;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(&account_address).await;

        let height = match db_response {
            Ok(response) => response.data.as_ref().ok_or(Status::new(tonic::Code::Internal, "missing Block data"))?.height,
            Err(storage::DatabaseError::NoLastBlock) => 0,
            _ => return Err(Status::new(tonic::Code::Internal, "couldn't get last block")),
          };

        Ok(Response::new(NextBlockHeightReply {next_height: height}))
    }
}