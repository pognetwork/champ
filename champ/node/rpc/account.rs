use crate::consensus::voting_power::get_actual_power;
use crate::state::ChampStateMutex;
pub use pog_proto::rpc::account_server::{Account, AccountServer};
use pog_proto::rpc::{BalanceReply, BalanceRequest, VotingPowerReply, VotingPowerRequest};
use pog_proto::rpc::{BlockByIdReply, BlockByIdRequest, BlockHeightReply, BlockHeightRequest};

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
        let response = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        match &response.data {
            Some(data) => Ok(Response::new(BalanceReply { balance: data.balance })),
            None => Err(Status::new(tonic::Code::Internal, "missing Block data")),
        }
    }
    async fn get_block_height(
        &self,
        block_height_request: Request<BlockHeightRequest>,
    ) -> Result<Response<BlockHeightReply>, Status> {
        let request = block_height_request.into_inner();
        let account_address = request.address;
        let get_next_block_height = request.get_next.unwrap_or(false) as u64;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(&account_address).await;

        let height = match db_response {
            Ok(response) => {
                response
                    .data
                    .as_ref()
                    .ok_or_else(|| Status::new(tonic::Code::Internal, "missing Block data"))?
                    .height
            }
            Err(storage::DatabaseError::NoLastBlock) => 0,
            _ => return Err(Status::new(tonic::Code::Internal, "couldn't get last block")),
        };

        Ok(Response::new(BlockHeightReply {
            next_height: height + get_next_block_height,
        }))
    }
    async fn get_voting_power(
        &self,
        request: Request<VotingPowerRequest>,
    ) -> Result<Response<VotingPowerReply>, Status> {
        let state = &self.state;
        // TODO: Change this return the Actual and Aktive voting power
        let power_result = get_actual_power(state, request.into_inner().address).await;
        let power = power_result.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;
        Ok(Response::new(VotingPowerReply { power: power }))
    }
    async fn get_block_by_id(&self, request: Request<BlockByIdRequest>) -> Result<Response<BlockByIdReply>, Status> {
        let block_hash = request.into_inner().hash;

        let state = self.state.lock().await;
        let db_response = state.db.get_block_by_id(&hex::encode(block_hash)).await;
        let block = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(BlockByIdReply {
            block: Some(block.to_owned()),
        }))
    }
}
