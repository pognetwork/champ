use std::convert::TryInto;

use crate::state::ChampStateMutex;
use anyhow::Result;
use crypto::{self, curves::curve25519::verify_signature};
use encoding::account::generate_account_address;
use pog_proto::api::{
    transaction::{Data, TxClaim},
    Block,
};
use prost::Message;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Validation {
    #[error("transactions could not be validated")]
    TxValidationError,
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
}

#[derive(Error, Debug)]
pub enum Node {
    #[error("block data not found")]
    BlockDataNotFound,
    #[error("block not found")]
    BlockNotFound,
    #[error{"tx not found"}]
    TxNotFound,
}

// Validate block
#[allow(dead_code)]
pub async fn validate(block: &Block, state: &ChampStateMutex) -> Result<()> {
    let data = block.clone().data.ok_or(Node::BlockDataNotFound)?;
    let public_key = &block.public_key;
    let signature = &block.signature;
    let db = &state.lock().await.db;
    let account_id = generate_account_address(public_key.to_vec())?;

    let response = db.get_latest_block_by_account(account_id).await;

    let latest_block = match response {
        Ok(block) => block,
        Err(storage::DatabaseError::NoLastBlock) => return verify_account_genesis_block(),
        _ => return Err(Node::BlockNotFound.into()),
    };

    // signature
    verify_signature(&data.encode_to_vec(), public_key, signature)?;
    // height / previous block
    verify_previous_block(block, latest_block)?;
    // transactions / balance
    verify_transactions(block, latest_block, state).await?;

    Ok(())
}

// TODO: add own error type to not disrupt the program
// Verifies the transactions and balances
async fn verify_transactions(new_block: &Block, prev_block: &Block, state: &ChampStateMutex) -> Result<()> {
    // go through all tx in the block and do math to see new balance
    // check against block balance
    let new_data = new_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;
    let prev_data = prev_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;

    let mut new_balance: i128 = prev_data.balance as i128;
    for transaction in &new_data.transactions {
        let tx_type = transaction.data.as_ref().ok_or(Validation::TransactionDataNotFound)?;
        new_balance += match tx_type {
            Data::TxSend(t) => -(t.amount as i128), // remove money from this balance
            Data::TxCollect(t) => validate_collect(t, state).await?,
            _ => new_balance,
        };
    }

    if new_balance == new_data.balance as i128 && new_balance > 0 {
        return Ok(());
    }

    Err(Validation::TxValidationError.into())
}

// Verifies the block height and previous block
fn verify_previous_block(new_block: &Block, prev_block: &Block) -> Result<()> {
    let new_data = new_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;
    let prev_data = prev_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;

    if new_data.height - 1 != prev_data.height {
        return Err(Validation::BlockHeightError.into());
    }
    if new_data.previous != Some(prev_block.get_id()?.to_vec()) {
        return Err(Validation::PreviousBlockError.into());
    }
    Ok(())
}

// Verifies the block height and previous block
fn verify_account_genesis_block() -> Result<()> {
    unimplemented!()
}

async fn validate_collect(tx: &TxClaim, state: &ChampStateMutex) -> Result<i128> {
    // check DB for send with id tx_id
    let db = &state.lock().await.db;
    let tx_id = match tx.transaction_id.clone().try_into() {
        Ok(a) => a,
        Err(_) => return Err(Node::TxNotFound.into()),
    };
    let db_response = db.get_transaction_by_id(tx_id).await;
    let transaction = match db_response {
        Ok(t) => t,
        Err(_) => return Err(Validation::TxValidationError.into()),
    };

    match &transaction.data {
        Some(Data::TxSend(t)) => Ok(t.amount.into()),
        Some(_) => Err(Validation::MissmatchedTx.into()),
        None => Err(Validation::SendTxNotFound.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::validation::block::{Node, verify_previous_block, verify_transactions};
    use crate::ChampState;
    use anyhow::Result;
    use pog_proto::api::transaction::TxClaim;
    use pog_proto::api::{
        block::BlockData,
        transaction::{Data, TxSend},
        Block, Transaction,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_verify_previous_block() -> Result<()> {
        let prev_block = Block {
            signature: b"thisIsNewSignature".to_vec(),
            public_key: b"someOtherKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 100,
                height: 4,
                previous: Some(b"blockBeforeMe".to_vec()),
                transactions: [].to_vec(),
            }),
        };
        let new_block = Block {
            signature: b"signedByMe".to_vec(),
            public_key: b"someKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 100,
                height: 5,
                previous: Some(prev_block.get_id()?.to_vec()),
                transactions: [].to_vec(),
            }),
        };

        //assert_eq!(verify_previous_block(&new_block, &prev_block)?, ());
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_transactions() -> Result<()> {
        let prev_block = Block {
            signature: b"thisIsNewSignature".to_vec(),
            public_key: b"someOtherKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 100,
                height: 4,
                previous: Some(b"blockBeforeMe".to_vec()),
                transactions: [].to_vec(),
            }),
        };
        let block = Block {
            signature: b"signedByMe".to_vec(),
            public_key: b"someKey".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 40,
                height: 5,
                previous: Some(prev_block.get_id()?.to_vec()),
                transactions: [
                    Transaction {
                        data: Some(Data::TxSend(TxSend {
                            receiver: b"somereceiver".to_vec(),
                            amount: 10,
                            data: [].to_vec(),
                        })),
                    },
                    Transaction {
                        data: Some(Data::TxSend(TxSend {
                            receiver: b"somereceiver".to_vec(),
                            amount: 50,
                            data: [].to_vec(),
                        })),
                    },
                ]
                .to_vec(),
            }),
        };
        let check_claim_tx = Transaction {
            data: Some(Data::TxSend(TxSend {
                receiver: b"somereceiver".to_vec(),
                amount: 10,
                data: [].to_vec(),
            })),
        };
        let data_block_1 = Block {
            signature: b"data_block_one".to_vec(),
            public_key: b"key_one".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 40,
                height: 5,
                previous: Some(prev_block.get_id()?.to_vec()),
                transactions: [check_claim_tx.clone()].to_vec(),
            }),
        };
        let check_claim_previous = Block {
            signature: b"data_block_one".to_vec(),
            public_key: b"key_one".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 40,
                height: 5,
                previous: Some(b"some_previous".to_vec()),
                transactions: [Transaction {
                    data: Some(Data::TxSend(TxSend {
                        receiver: b"somereceiver".to_vec(),
                        amount: 10,
                        data: [].to_vec(),
                    })),
                }]
                .to_vec(),
            }),
        };
        let check_claim = Block {
            signature: b"data_block_one".to_vec(),
            public_key: b"key_one".to_vec(),
            timestamp: 1,
            data: Some(BlockData {
                version: 1,
                signature_type: 1,
                balance: 40,
                height: 5,
                previous: Some(check_claim_previous.get_id()?.to_vec()),
                transactions: [Transaction {
                    data: Some(Data::TxCollect(TxClaim {
                        transaction_id: check_claim_tx.get_id(data_block_1.get_id()?)?.to_vec(),
                    })),
                }]
                .to_vec(),
            }),
        };
        // check if this works
        let db = storage::new(&storage::DatabaseConfig {
            kind: storage::Databases::Mock,
            uri: "",
        })
        .await?;

        let state = Arc::new(Mutex::new(ChampState {
            db,
        }));

        // Populate the DB with dummy data
        let database = &mut state.lock().await.db;
        database.add_block(data_block_1).await?;

        assert_eq!(verify_transactions(&block, &prev_block, &state).await?, ());
        assert_eq!(verify_transactions(&check_claim, &check_claim_previous, &state).await?, ());

        Ok(())
    }
}
