use std::convert::TryInto;

use crate::consensus::voting_power::{get_active_power, get_actual_power};
use crate::state::ChampStateArc;
use crate::storage;
use crate::validation::block::validate;

use pog_proto::api::{self, SignedBlock};
use pog_proto::rpc::lattice::*;

pub use pog_proto::rpc::lattice::lattice_server::{Lattice, LatticeServer};

use tonic::{Request, Response, Status};
use tracing::debug;
#[derive(Debug)]
pub struct LatticeService {
    pub state: ChampStateArc,
}

impl LatticeService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

#[tonic::async_trait]
impl Lattice for LatticeService {
    async fn get_unclaimed_transactions(
        &self,
        request: Request<GetUnclaimedTransactionsRequest>,
    ) -> Result<Response<GetUnclaimedTransactionsReply>, Status> {
        let req = request.into_inner();
        let db = self.state.db.lock().await;

        let addr = match api::AccountID::try_from(req.address) {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db_response = db.get_unclaimed_transactions(addr).await;
        let response = db_response.map_err(|_| Status::new(tonic::Code::Internal, "internal server error"))?;

        let mut txs = vec![];
        for (transaction_id, transaction) in response {
            txs.push(Tx {
                transaction: Some(transaction),
                transaction_id: transaction_id.to_vec(),
            })
        }

        Ok(Response::new(GetUnclaimedTransactionsReply {
            data: txs,
        }))
    }

    async fn get_balance(&self, request: Request<BalanceRequest>) -> Result<Response<BalanceReply>, Status> {
        // We must use .into_inner() as the fields of gRPC requests and responses are private
        let address: api::AccountID = match request.into_inner().address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db = self.state.db.lock().await;
        let db_response = db.get_latest_block_by_account(address).await;
        let response = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(BalanceReply {
            balance: response.data.balance,
        }))
    }

    async fn get_latest_block(
        &self,
        request: Request<LatestBlockRequest>,
    ) -> Result<Response<LatestBlockReply>, Status> {
        let request = request.into_inner();

        let address: api::AccountID = match request.address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let db = self.state.db.lock().await;
        let db_response = db.get_latest_block_by_account(address).await;

        let block = match db_response {
            Ok(response) => Some(response.into()),
            Err(storage::DatabaseError::NoLastBlock) => None,
            _ => return Err(Status::new(tonic::Code::Internal, "couldn't get last block")),
        };

        Ok(Response::new(LatestBlockReply {
            block,
        }))
    }

    /// returns the active voting power (with delegate power)
    async fn get_voting_power(
        &self,
        rpc_request: Request<VotingPowerRequest>,
    ) -> Result<Response<VotingPowerReply>, Status> {
        debug!("getting voting power");

        let state = &self.state;
        let request = rpc_request.into_inner().clone();

        let address: api::AccountID = match request.address.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };

        let power_result = match request.get_active {
            true => get_active_power(state, address).await,
            false => get_actual_power(state, address).await,
        };

        let power = power_result.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;
        Ok(Response::new(VotingPowerReply {
            power,
        }))
    }

    async fn get_block_by_id(&self, request: Request<BlockByIdRequest>) -> Result<Response<BlockByIdReply>, Status> {
        debug!("getting block by id");

        let block_id: api::BlockID = request
            .into_inner()
            .hash
            .try_into()
            .map_err(|_| Status::new(tonic::Code::Internal, "couldn't parse address"))?;

        let db = self.state.db.lock().await;
        let db_response = db.get_block_by_id(block_id).await;
        let block = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(BlockByIdReply {
            block: Some(block.into()),
        }))
    }

    async fn get_delegate(
        &self,
        request: tonic::Request<DelegateRequest>,
    ) -> Result<tonic::Response<DelegateReply>, tonic::Status> {
        debug!("getting delegate of an account");

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

    async fn get_blocks(
        &self,
        request: tonic::Request<GetBlocksRequest>,
    ) -> Result<tonic::Response<GetBlocksReply>, tonic::Status> {
        let req = request.into_inner();
        let db = self.state.db.lock().await;

        let address: Option<api::AccountID> = match req.address {
            Some(addr) => match api::AccountID::try_from(addr) {
                Ok(a) => Some(a),
                Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
            },
            None => None,
        };

        if req.limit > 100 {
            return Err(Status::new(tonic::Code::Internal, "limit has to be < 100"));
        }

        let db_response = db.get_blocks(req.sort_by == 0, req.limit, req.offset, address).await;
        let response = db_response.map_err(|_| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(GetBlocksReply {
            blocks: response.iter().map(|b| b.to_owned().into()).collect(),
        }))
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
        debug!("getting transaction by id");

        let transaction_id: api::TransactionID = match request.into_inner().transaction_id.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Status::new(tonic::Code::Internal, "Address could not be parsed")),
        };
        let db = self.state.db.lock().await;
        let db_response = db.get_transaction_by_id(transaction_id).await;
        let (transaction, block, address) =
            db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;

        Ok(Response::new(TxByIdReply {
            transaction: Some(transaction),
            block: block.to_vec(),
            address: address.to_vec(),
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

    async fn submit_block(
        &self,
        request: tonic::Request<pog_proto::api::RawBlock>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let block = request.into_inner();
        let block: SignedBlock =
            block.try_into().map_err(|_e| Status::new(tonic::Code::Internal, "invalid block: encoding"))?;

        let internal_config = { self.state.config.read().await.internal.clone() };
        if internal_config.debug_skip_consensus {
            let mut db = self.state.db.lock().await;

            if !internal_config.debug_skip_block_validation && validate(&block, &self.state).await.is_err() {
                return Err(Status::new(tonic::Code::Internal, "invalid block: validation"));
            }

            let db_response = db.add_block(block).await;
            let _ = db_response.map_err(|_e| Status::new(tonic::Code::Internal, "internal server error"))?;
        }

        Ok(Response::new(Empty {}))
    }

    async fn get_block_spam_index(
        &self,
        _request: tonic::Request<pog_proto::api::RawBlock>,
    ) -> Result<tonic::Response<BlockSpamIndexReply>, tonic::Status> {
        unimplemented!()
    }
}
