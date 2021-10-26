use std::convert::TryInto;

use crate::consensus::voting_power::{get_active_power, get_actual_power};
use crate::state::ChampStateArc;
use crate::storage;

use pog_proto::api;
use pog_proto::rpc::block::*;

pub use pog_proto::rpc::block::block_server::{Block, BlockServer};

use tonic::{Request, Response, Status};
#[derive(Debug)]
pub struct BlockService {
    pub state: ChampStateArc,
}

impl BlockService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

#[tonic::async_trait]
impl Block for BlockService {
    async fn get_balance(&self, request: Request<BalanceRequest>) -> Result<Response<BalanceReply>, Status> {
        // We must use .into_inner() as the fields of gRPC requests and responses are private
        let address: api::AccountID = match request.into_inner().address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db = self.state.db.lock().await;
        let db_response = db.get_latest_block_by_account(address).await;
        let response = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        match &response.data {
            Some(data) => Ok(Response::new(BalanceReply {
                balance: data.balance,
            })),
            None => Err(Status::new(tonic::Code::Internal, "missing Block data")),
        }
    }

    async fn get_block_height(
        &self,
        block_height_request: Request<BlockHeightRequest>,
    ) -> Result<Response<BlockHeightReply>, Status> {
        let request = block_height_request.into_inner();
        let get_next_block_height = request.get_next.unwrap_or(false) as u64;

        let address: api::AccountID = match request.address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db = self.state.db.lock().await;
        let db_response = db.get_latest_block_by_account(address).await;

        let height = match db_response {
            Ok(response) => {
                response.data.as_ref().ok_or_else(|| Status::new(tonic::Code::Internal, "missing Block data"))?.height
            }
            Err(storage::DatabaseError::NoLastBlock) => 0,
            _ => return Err(Status::new(tonic::Code::Internal, "couldn't get last block")),
        };

        Ok(Response::new(BlockHeightReply {
            next_height: height + get_next_block_height,
        }))
    }

    /// returns the active voting power (with delegate power)
    async fn get_voting_power(
        &self,
        rpc_request: Request<VotingPowerRequest>,
    ) -> Result<Response<VotingPowerReply>, Status> {
        let state = &self.state;
        let request = rpc_request.into_inner().clone();

        let address: api::AccountID = match request.address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let power_result = match request.get_active.unwrap_or(false) {
            true => get_active_power(state, address).await,
            false => get_actual_power(state, address).await,
        };

        let power = power_result.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;
        Ok(Response::new(VotingPowerReply {
            power,
        }))
    }

    async fn get_block_by_id(&self, request: Request<BlockByIdRequest>) -> Result<Response<BlockByIdReply>, Status> {
        let block_id: api::BlockID = request
            .into_inner()
            .hash
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        let db = self.state.db.lock().await;
        let db_response = db.get_block_by_id(block_id).await;
        let block = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(BlockByIdReply {
            block: Some(block.to_owned()),
        }))
    }

    async fn get_delegate(
        &self,
        request: tonic::Request<DelegateRequest>,
    ) -> Result<tonic::Response<DelegateReply>, tonic::Status> {
        let db = self.state.db.lock().await;

        let address: api::AccountID = match request.into_inner().address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db_response = db.get_account_delegate(address).await;
        let response = db_response.map_err(|_| Status::new(tonic::Code::Internal, "internal server error"))?;

        match &response {
            Some(address) => Ok(Response::new(DelegateReply {
                delegate_address: address.to_vec(),
            })),
            None => Err(Status::new(tonic::Code::Internal, "missing Block data")),
        }
    }

    async fn get_pending_blocks(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<PendingBlockReply>, tonic::Status> {
        unimplemented!()
    }

    async fn get_unacknowledged_tx(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<UnacknowledgedTxReply>, tonic::Status> {
        unimplemented!()
    }

    async fn get_tx_by_id(
        &self,
        request: tonic::Request<TxByIdRequest>,
    ) -> Result<tonic::Response<TxByIdReply>, tonic::Status> {
        let transaction_id: api::TransactionID = match request.into_inner().transaction_id.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };
        let db = self.state.db.lock().await;
        let db_response = db.get_transaction_by_id(transaction_id).await;
        let transaction = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(TxByIdReply {
            transaction: Some(transaction.to_owned()),
        }))
    }

    async fn get_tx_by_index(
        &self,
        _request: tonic::Request<TxByIndexRequest>,
    ) -> Result<tonic::Response<TxByIndexReply>, tonic::Status> {
        // get blocks where type is tx
        // use index to get tx inside a block
        unimplemented!()
    }
}
