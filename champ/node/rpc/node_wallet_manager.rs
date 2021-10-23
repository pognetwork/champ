use pog_proto::rpc::node_wallet_manager::node_wallet_manager_server::NodeWalletManager;
use pog_proto::rpc::node_wallet_manager::{
    AddWalletReply, AddWalletRequest, DecryptMessageReply, DecryptMessageRequest, Empty, EncryptMessageReply,
    EncryptMessageRequest, GetWalletReply, GetWalletRequest, GetWalletsReply, RemoveWalletRequest, SignBlockReply,
    SignBlockRequest, SignMessageReply, SignMessageRequest, VerifySignatureRequest,
};

use crate::state::ChampStateArc;

#[derive(Debug)]
pub struct NodeWalletManagerService {
    pub state: ChampStateArc,
}

impl NodeWalletManagerService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

#[tonic::async_trait]
impl NodeWalletManager for NodeWalletManagerService {
    async fn get_wallets(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetWalletsReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_wallet(
        &self,
        _request: tonic::Request<GetWalletRequest>,
    ) -> Result<tonic::Response<GetWalletReply>, tonic::Status> {
        unimplemented!()
    }
    async fn add_wallet(
        &self,
        _request: tonic::Request<AddWalletRequest>,
    ) -> Result<tonic::Response<AddWalletReply>, tonic::Status> {
        unimplemented!()
    }
    async fn remove_wallet(
        &self,
        _request: tonic::Request<RemoveWalletRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        unimplemented!()
    }
    async fn sign_message(
        &self,
        _request: tonic::Request<SignMessageRequest>,
    ) -> Result<tonic::Response<SignMessageReply>, tonic::Status> {
        unimplemented!()
    }
    async fn sign_block(
        &self,
        _request: tonic::Request<SignBlockRequest>,
    ) -> Result<tonic::Response<SignBlockReply>, tonic::Status> {
        unimplemented!()
    }
    async fn verify_signature(
        &self,
        _request: tonic::Request<VerifySignatureRequest>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        unimplemented!()
    }
    async fn encrypt_message(
        &self,
        _request: tonic::Request<EncryptMessageRequest>,
    ) -> Result<tonic::Response<EncryptMessageReply>, tonic::Status> {
        unimplemented!()
    }
    async fn decrypt_message(
        &self,
        _request: tonic::Request<DecryptMessageRequest>,
    ) -> Result<tonic::Response<DecryptMessageReply>, tonic::Status> {
        unimplemented!()
    }
}
