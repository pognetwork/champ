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
}

#[derive(Error, Debug)]
pub enum Node {
    #[error("block data not found")]
    BlockDataNotFound,
    #[error("block not found")]
    BlockNotFound,
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
    verify_transactions(block, latest_block)?;

    Ok(())
}

// TODO: add own error type to not disrupt the program
// Verifies the transactions and balances
fn verify_transactions(new_block: &Block, prev_block: &Block) -> Result<()> {
    // go through all tx in the block and do math to see new balance
    // check against block balance
    let new_data = new_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;
    let prev_data = prev_block.data.as_ref().ok_or(Node::BlockDataNotFound)?;

    let mut new_balance: i128 = prev_data.balance as i128;
    for transaction in &new_data.transactions {
        let tx_type = transaction.data.as_ref().ok_or(Validation::TransactionDataNotFound)?;
        new_balance += match tx_type {
            Data::TxSend(t) => -(t.amount as i128), // remove money from this balance
            Data::TxCollect(t) => validate_collect(t),
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

fn validate_collect(_tx: &TxClaim) -> i128 {
    // check send blocks where receiver = this account_id and tx_id is send block id
    // check block has not already been claimed
    // add money to the balance

    unimplemented!()
}

#[cfg(test)]
mod tests {
    use crate::validation::block::{verify_previous_block, verify_transactions};
    use anyhow::Result;
    use pog_proto::api::{
        block::BlockData,
        transaction::{Data, TxSend},
        Block, Transaction,
    };

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

        assert_eq!(verify_previous_block(&new_block, &prev_block)?, ());
        Ok(())
    }

    #[test]
    fn test_verify_transactions() -> Result<()> {
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

        assert_eq!(verify_transactions(&block, &prev_block)?, ());
        Ok(())
    }
}
