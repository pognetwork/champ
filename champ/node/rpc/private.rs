use pog_proto::rpc::private::{
    private_server::Private, AddAccountReply, AddAccountRequest, DecryptMessageReply, DecryptMessageRequest, Empty,
    EncryptMessageReply, EncryptMessageRequest, GetAccountReply, GetAccountRequest, GetAccountsReply,
    RemoveAccountRequest, SignBlockReply, SignBlockRequest, SignMessageReply, SignMessageRequest,
    VerifySignatureRequest,
};

use crate::state::ChampStateArc;

#[derive(Debug)]
pub struct PrivateService {
    pub state: ChampStateArc,
}

impl PrivateService {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }
}

#[tonic::async_trait]
impl Private for PrivateService {
    async fn get_accounts(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<GetAccountsReply>, tonic::Status> {
        unimplemented!()
    }
    async fn get_account(
        &self,
        _request: tonic::Request<GetAccountRequest>,
    ) -> Result<tonic::Response<GetAccountReply>, tonic::Status> {
        unimplemented!()
    }
    async fn add_account(
        &self,
        _request: tonic::Request<AddAccountRequest>,
    ) -> Result<tonic::Response<AddAccountReply>, tonic::Status> {
        unimplemented!()
    }
    async fn remove_account(
        &self,
        _request: tonic::Request<RemoveAccountRequest>,
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
