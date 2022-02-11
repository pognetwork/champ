use crate::storage;
use crate::{state::ChampStateArc, storage::DatabaseError};

use crypto::{self, signatures::ed25519::verify_signature};
use encoding::account::{generate_account_address, validate_account_address};
use pog_proto::api::{
    transaction::{Data, TxClaim, TxSend},
    SignedBlock, Transaction,
};
use prost::Message;
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{debug, trace};

#[derive(Error, Debug)]
pub enum Validation {
    #[error("transactions could not be validated {0}")]
    TxValidationError(String),
    #[error("invalid block height")]
    BlockHeightError,
    #[error("previous block did not match")]
    PreviousBlockError,
    #[error("transaction data not found")]
    TransactionDataNotFound,
    #[error("corresponding send tx not found")]
    SendTxNotFound,
    #[error("attempt to claim non send tx")]
    MissmatchedTx,
    #[error("duplicate transaction")]
    DuplicatedTx,
    #[error("too many transactions")]
    TooManyTransactions,
    #[error("send receiver cannot be block account")]
    ReceiverAccountError,
    #[error("block already exists")]
    BlockDuplicate,
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Node {
    #[error("block data not found")]
    BlockDataNotFound,
    #[error("block not found")]
    BlockNotFound,
    #[error{"tx not found"}]
    TxNotFound,
    #[error{"unknown crypto error"}]
    CryptoError,
    #[error{"error generating account address"}]
    AccountError,
    #[error{"db error: {0}"}]
    DBError(DatabaseError),
    #[error{"async error"}]
    AsyncError,
    #[error{"block id could not be created"}]
    BlockIdError,
}

#[derive(Error, Debug)]
pub enum BlockValidationError {
    #[error(transparent)]
    Invalid(#[from] Validation),
    #[error(transparent)]
    Error(#[from] Node),
}

// Validate block
#[allow(dead_code)]
#[tracing::instrument]
pub async fn validate(block: &SignedBlock, state: &ChampStateArc) -> Result<(), BlockValidationError> {
    debug!("validating a block");

    let data = block.clone().data.ok_or(Node::BlockDataNotFound)?;
    let public_key = &block.public_key;
    let signature = &block.signature;
    let db = &state.db.lock().await;
    let account_id = generate_account_address(public_key.to_vec()).map_err(|_| Node::CryptoError)?;

    let response = db.get_latest_block_by_account(account_id).await;

    let latest_block = match response {
        Ok(block) => block,
        Err(storage::DatabaseError::NoLastBlock) => return verify_account_genesis_block(block),
        _ => return Err(Node::BlockNotFound.into()),
    };

    let id = block.get_id().map_err(|_| Node::BlockIdError)?;
    match db.get_block_by_id(id).await {
        Ok(_) => return Err(Validation::BlockDuplicate.into()),
        Err(storage::DatabaseError::BlockNotFound) => (),
        Err(e) => return Err(Node::DBError(e).into()),
    }

    // signature
    verify_signature(&data.encode_to_vec(), public_key, signature).map_err(|_| Node::CryptoError)?;
    // height / previous block
    verify_previous_block(block, &latest_block)?;
    // transactions / balance
    verify_transactions(block, &latest_block, state).await?;

    trace!("Block successfully validated. Block={:?}", block);

    Ok(())
}

// TODO: add error handling so validation error go to voting
// Verifies the transactions and balances
async fn verify_transactions(
    new_block: &SignedBlock,
    prev_block: &SignedBlock,
    state: &ChampStateArc,
) -> Result<(), BlockValidationError> {
    debug!("verify transactions");
    // go through all tx in the block and do math to see new balance
    // check against block balance
    let new_data = new_block.data.clone().ok_or(Node::BlockDataNotFound)?;
    let prev_data = prev_block.data.clone().ok_or(Node::BlockDataNotFound)?;

    let mut transaction_ids: Vec<[u8; 32]> = vec![];

    if new_data.transactions.len() > 255 {
        return Err(Validation::TooManyTransactions.into());
    }

    let mut new_balance: i128 = prev_data.balance as i128;
    let mut tokio_tasks: Vec<JoinHandle<Result<i128, BlockValidationError>>> = vec![];

    for transaction in &new_data.transactions {
        // validate that transaction is not duplicated
        let blockid = new_block.get_id().map_err(|_| Node::BlockNotFound)?;
        let txid = transaction.get_id(blockid).map_err(|_| Node::TxNotFound)?;
        if transaction_ids.contains(&txid) {
            return Err(Validation::DuplicatedTx.into());
        }
        transaction_ids.push(txid);

        let s = state.clone();
        let tx = transaction.clone();
        let block = new_block.clone();
        let balance = new_balance;
        // concurrent verification
        let task: JoinHandle<Result<i128, BlockValidationError>> =
            tokio::spawn(async move { tx_verification(&s, block, &balance, &tx).await });
        tokio_tasks.push(task);
    }

    for t in tokio_tasks {
        new_balance += t.await.map_err(|_| Node::AsyncError)??;
    }

    if new_balance == new_data.balance as i128 && new_balance >= 0 {
        return Ok(());
    }

    Err(Validation::TxValidationError("verify transactions".to_string()).into())
}

async fn tx_verification(
    state: &ChampStateArc,
    new_block: SignedBlock,
    new_balance: &i128,
    transaction: &Transaction,
) -> Result<i128, BlockValidationError> {
    // calculate the new balance after all transactions are processed
    let tx_type = transaction.data.as_ref().ok_or(Validation::TransactionDataNotFound)?;

    let result_balance = match tx_type {
        Data::TxSend(tx) => validate_send(tx.amount, tx, new_block)?,
        Data::TxCollect(tx) => validate_collect(state, tx, &new_block).await?,
        _ => *new_balance,
    };
    Ok(result_balance)
}

// Verifies the block height and previous block
fn verify_previous_block(new_block: &SignedBlock, prev_block: &SignedBlock) -> Result<(), BlockValidationError> {
    debug!("verify previous block");
    let new_data = new_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;
    let prev_data = prev_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;

    if new_data.height - 1 != prev_data.height {
        return Err(Validation::BlockHeightError.into());
    }
    if new_data.previous != prev_block.get_id().map_err(|_| Node::BlockNotFound)?.to_vec() {
        return Err(Validation::PreviousBlockError.into());
    }
    Ok(())
}

// Verifies the block height and previous block
fn verify_account_genesis_block(block: &SignedBlock) -> Result<(), BlockValidationError> {
    let data = block.data.clone().ok_or(BlockValidationError::Error(Node::BlockDataNotFound))?;

    if data.height != 0 {
        return Err(Validation::BlockHeightError.into());
    }

    Ok(())
}

fn validate_send(amount: u64, tx: &TxSend, new_block: SignedBlock) -> Result<i128, BlockValidationError> {
    let receiver = tx.receiver.clone();
    if receiver == generate_account_address(new_block.public_key).map_err(|_| Node::AccountError)? {
        return Err(Validation::ReceiverAccountError.into());
    }

    validate_account_address(receiver).map_err(|_| Validation::ReceiverAccountError)?;

    Ok(-(amount as i128))
}

#[allow(clippy::borrowed_box)]
async fn validate_collect(
    state: &ChampStateArc,
    tx: &TxClaim,
    block: &SignedBlock,
) -> Result<i128, BlockValidationError> {
    debug!("verify claim transactions");
    let send_id = match tx.transaction_id.clone().try_into() {
        Ok(a) => a,
        Err(_) => return Err(Node::TxNotFound.into()),
    };

    let db = &state.db.lock().await;
    let resp = db.get_send_recipient(send_id).await;
    if resp.map_err(|e| Node::DBError(e))?.is_some() {
        return Err(Validation::TxValidationError("validate collect 1".to_string()).into());
    }

    let db_response = db.get_transaction_by_id(send_id).await;
    let receive_tx = match db_response {
        Ok(t) => t,
        Err(_) => return Err(Validation::TxValidationError("validate collect 2".to_string()).into()),
    };

    let sendtx = match &receive_tx.data {
        Some(Data::TxSend(t)) => t,
        Some(_) => return Err(Validation::MissmatchedTx.into()),
        None => return Err(Validation::SendTxNotFound.into()),
    };

    let account_id = generate_account_address(block.public_key.to_vec()).map_err(|_| Node::AccountError)?;
    // check if account is allowed to receive
    if account_id.to_vec() != sendtx.receiver {
        return Err(Validation::TxValidationError("validate collect 3".to_string()).into());
    }

    Ok(sendtx.amount.into())
}

#[cfg(test)]
mod tests {
    use crate::validation::block::{verify_previous_block, verify_transactions};
    use crate::ChampState;
    use anyhow::Result;
    use encoding::zbase32::FromZbase;
    use pog_proto::api::transaction::TxClaim;
    use pog_proto::api::{
        signed_block::BlockData,
        transaction::{Data, TxSend},
        SignedBlock, Transaction,
    };

    #[test]
    fn test_verify_previous_block() -> Result<()> {
        let prev_block = SignedBlock {
            signature: b"thisIsNewSignature".to_vec(),
            public_key: b"someOtherKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 100,
                height: 4,
                previous: b"blockBeforeMe".to_vec(),
                transactions: [].to_vec(),
            }),
        };
        let new_block = SignedBlock {
            signature: b"signedByMe".to_vec(),
            public_key: b"someKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 100,
                height: 5,
                previous: prev_block.get_id().expect("get Block ID").to_vec(),
                transactions: [].to_vec(),
            }),
        };

        assert!(verify_previous_block(&new_block, &prev_block).is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_transactions() -> Result<()> {
        let prev_block = SignedBlock {
            signature: b"thisIsNewSignature".to_vec(),
            public_key: b"someOtherKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 100,
                height: 4,
                previous: b"blockBeforeMe".to_vec(),
                transactions: vec![],
            }),
        };
        let block = SignedBlock {
            signature: b"signedByMe".to_vec(),
            public_key: b"someKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 40,
                height: 5,
                previous: prev_block.get_id().expect("get Block ID").to_vec(),
                transactions: vec![
                    Transaction {
                        data: Some(Data::TxSend(TxSend {
                            receiver: Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap(),
                            amount: 10,
                            data: [].to_vec(),
                        })),
                    },
                    Transaction {
                        data: Some(Data::TxSend(TxSend {
                            receiver: Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap(),
                            amount: 50,
                            data: [].to_vec(),
                        })),
                    },
                ],
            }),
        };
        let check_claim_tx = Transaction {
            data: Some(Data::TxSend(TxSend {
                receiver: Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap(),
                amount: 10,
                data: vec![],
            })),
        };
        let data_block_1 = SignedBlock {
            signature: b"data_block_one".to_vec(),
            public_key: b"key_one".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 40,
                height: 0,
                previous: prev_block.get_id().expect("get Block ID").to_vec(),
                transactions: vec![check_claim_tx.clone()],
            }),
        };
        let check_claim_previous = SignedBlock {
            signature: b"data_block_one".to_vec(),
            public_key: b"key_one".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 40,
                height: 5,
                previous: b"some_previous".to_vec(),
                transactions: vec![],
            }),
        };
        let check_claim = SignedBlock {
            signature: b"data_block_one".to_vec(),
            public_key: b"test".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 0,
                signature_type: 0,
                balance: 50,
                height: 6,
                previous: check_claim_previous.get_id().expect("get block ID").to_vec(),
                transactions: vec![Transaction {
                    data: Some(Data::TxCollect(TxClaim {
                        transaction_id: check_claim_tx
                            .get_id(data_block_1.get_id().expect("get block ID"))
                            .expect("get Tx ID")
                            .to_vec(),
                    })),
                }],
            }),
        };

        let state = ChampState::mock().await;
        state.db.lock().await.add_block(data_block_1).await.expect("block should be added");
        verify_transactions(&block, &prev_block, &state).await.expect("should work");
        assert_eq!(
            verify_transactions(&check_claim, &check_claim_previous, &state)
                .await
                .expect("tx should be verified. Tx Nr: 2"),
            ()
        );

        Ok(())
    }
}
