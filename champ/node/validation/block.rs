use crate::state::ChampStateMutex;
use anyhow::{anyhow, Result};
use crypto::{self, curves::curve25519::verify_signature};
use encoding::account::generate_account_address;
use pog_proto::api::{
    transaction::{Data, TxClaim, TxSend},
    AccountID, Block,
};
use prost::Message;

// Validate block
pub async fn validate(block: &Block, state: &ChampStateMutex) -> Result<()> {
    let data = block.clone().data.ok_or_else(|| anyhow!("block data not found"))?;
    let public_key = &block.public_key;
    let signature = &block.signature;
    let db = &state.lock().await.db;
    let account_id = generate_account_address(public_key.to_vec())?;

    if data.height == 0 {
        // if new_block is the first block in the chain
        verify_account_genesis_block(&block)?;
    }

    // signature
    verify_signature(&data.encode_to_vec(), public_key, signature)?;
    // height / previous block
    let prev_block_height = data.height - 1;
    let response = db.get_block_by_height(account_id, &prev_block_height).await;
    let prev_block_option = response.map_err(|_e| anyhow!("internal db error"))?;
    let prev_block = prev_block_option.ok_or_else(|| anyhow!("no block found"))?; //TODO: skip this block
    verify_previous_block(block, prev_block)?;
    // transactions / balance
    verify_transactions(block, prev_block)?;

    Ok(())
}

// TODO: add own error type to not disrupt the program
// Verifies the transactions and balances
fn verify_transactions(new_block: &Block, prev_block: &Block) -> Result<()> {
    // go through all tx in the block and do math to see new balance
    // check against block balance
    let new_data = new_block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;
    let prev_data = prev_block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;

    let mut new_balance = prev_data.balance;
    for transaction in &new_data.transactions {
        let tx_type = transaction.data.as_ref().ok_or_else(|| anyhow!("transaction data not found"))?;
        new_balance = match tx_type {
            Data::TxSend(t) => validate_send(t, new_balance as i64)?, // remove money from this balance TODO: validate sender
            Data::TxCollect(t) => validate_collect(t, new_balance)?,
            _ => new_balance,
        };
    }
    unimplemented!()
}

// Verifies the block height and previous block
fn verify_previous_block(new_block: &Block, prev_block: &Block) -> Result<()> {
    let new_data = new_block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;
    let prev_data = prev_block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;

    if new_data.height - 1 != prev_data.height {
        return Err(anyhow!("block height not match expected"));
    }
    if new_data.previous != Some(prev_block.get_id()?.to_vec()) {
        return Err(anyhow!("previous block did not match as expected"));
    }
    Ok(())
}

// Verifies the block height and previous block
fn verify_account_genesis_block(new_block: &Block) -> Result<()> {
    unimplemented!()
}

fn validate_send(tx: &TxSend, balance: i64) -> Result<u64> {
    let result: i64 = balance - tx.amount as i64;
    if result < 0 {
        return Err(anyhow!("balance cannot be negative"));
    }
    Ok(result as u64)
}

fn validate_collect(tx: &TxClaim, balance: u64) -> Result<u64> {
    unimplemented!()
}
