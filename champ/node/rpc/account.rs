use crate::state::ChampStateMutex;
pub use pog_proto::rpc::account_server::{Account, AccountServer};
use pog_proto::rpc::{BalanceReply, BalanceRequest, VotingPowerReply, VotingPowerRequest};
use pog_proto::rpc::{NextBlockHeightReply, NextBlockHeightRequest};

use derive_new::new;
use tonic::{Request, Response, Status};

#[derive(Debug, new)]
pub struct AccountService {
    pub state: ChampStateMutex,
}

#[tonic::async_trait]
impl Account for AccountService {
    async fn get_balance(&self, request: Request<BalanceRequest>) -> Result<Response<BalanceReply>, Status> {
        // We must use .into_inner() as the fields of gRPC requests and responses are private
        let account_address = request.into_inner().address;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(&account_address).await;
        let response =
            db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        match &response.data {
            Some(data) => Ok(Response::new(BalanceReply {
                balance: data.balance,
            })),
            None => Err(Status::new(tonic::Code::Internal, "missing Block data")),
        }
    }

    async fn get_next_block_height(
        &self,
        request: Request<NextBlockHeightRequest>,
    ) -> Result<Response<NextBlockHeightReply>, Status> {
        let account_address = request.into_inner().address;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(&account_address).await;

        let height = match db_response {
            Ok(response) => {
                response
                    .data
                    .as_ref()
                    .ok_or(Status::new(tonic::Code::Internal, "missing Block data"))?
                    .height
            }
            Err(storage::DatabaseError::NoLastBlock) => 0,
            _ => return Err(Status::new(tonic::Code::Internal, "couldn't get last block")),
        };

        Ok(Response::new(NextBlockHeightReply {
            next_height: height + 1,
        }))
    }

    async fn get_voting_power(
        &self,
        request: Request<VotingPowerRequest>,
    ) -> Result<Response<VotingPowerReply>, Status> {
        let account_address = request.into_inner().address;

        let state = self.state.lock().await;
        let db_response = state.db.get_account_by_id(&account_address).await;
        let response =
            db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(VotingPowerReply {
            power: response.voting_power,
        }))
    }
}
