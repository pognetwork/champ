use std::convert::TryInto;

use crate::consensus::voting_power::get_actual_power;
use crate::state::ChampStateMutex;
use encoding::zbase32::FromZbase;
use pog_proto::api;
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
        let address: api::AccountID = Vec::from_zbase(request.into_inner().address)
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(address).await;
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

        let address: api::AccountID = Vec::from_zbase(account_address)
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        let state = self.state.lock().await;
        let db_response = state.db.get_latest_block_by_account(address).await;

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

        let address: api::AccountID = Vec::from_zbase(request.into_inner().address)
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        // TODO: Change this return the Actual and active voting power
        let power_result = get_actual_power(state, address).await;
        let power = power_result.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;
        Ok(Response::new(VotingPowerReply { power }))
    }
    async fn get_block_by_id(&self, request: Request<BlockByIdRequest>) -> Result<Response<BlockByIdReply>, Status> {
        let block_id: api::BlockID = request
            .into_inner()
            .hash
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        let state = self.state.lock().await;
        let db_response = state.db.get_block_by_id(block_id).await;
        let block = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(BlockByIdReply {
            block: Some(block.to_owned()),
        }))
    }

    async fn get_delegate(
        &self,
        _request: tonic::Request<pog_proto::rpc::DelegateRequest>,
    ) -> Result<tonic::Response<pog_proto::rpc::DelegateReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_pending_blocks(
        &self,
        _request: tonic::Request<pog_proto::rpc::Empty>,
    ) -> Result<tonic::Response<pog_proto::rpc::PendingBlockReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_unacknowledged_tx(
        &self,
        _request: tonic::Request<pog_proto::rpc::Empty>,
    ) -> Result<tonic::Response<pog_proto::rpc::UnacknowledgedTxReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_tx_by_id(
        &self,
        _request: tonic::Request<pog_proto::rpc::TxByIdRequest>,
    ) -> Result<tonic::Response<pog_proto::rpc::TxByIdReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_tx_by_index(
        &self,
        _request: tonic::Request<pog_proto::rpc::TxByIndexRequest>,
    ) -> Result<tonic::Response<pog_proto::rpc::TxByIndexReply>, tonic::Status> {
        unimplemented!()
    }
}
