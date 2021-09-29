use crate::state::ChampStateMutex;
use anyhow::{anyhow, Result};
use crypto::{self, curves::curve25519::verify_signature};
use encoding::account::generate_account_address;
use pog_proto::api::Block;
use prost::Message;

// Validate block
pub async fn validate(block: &Block, state: &ChampStateMutex) -> Result<()> {
    let data = block.clone().data.ok_or_else(|| anyhow!("block data not found"))?;
    let public_key = &block.public_key;
    let signature = &block.signature;
    let db = &state.lock().await.db;
    let account_id = generate_account_address(public_key.to_vec())?;

    // signature
    verify_signature(&data.encode_to_vec(), public_key, signature)?;
    // transactions / balance
    // height / previous block
    let response = db.get_block_by_height(account_id, &data.height).await;
    let prev_block = response.map_err(|_e| anyhow!("internal db error"))?;
    verify_previous_block(block, prev_block)?;

    Ok(())
}

// Verifies the transactions and balances
fn verify_transactions() -> Result<()> {
    unimplemented!()
}

// Verifies the block height and previous block
fn verify_previous_block(new_block: &Block, prev_block: &Block) -> Result<()> {
    let new_data = new_block.data.as_ref().ok_or_else(|| anyhow!("block data not found"))?;
    let prev_data = prev_block
        .data
        .as_ref()
        .ok_or_else(|| anyhow!("block data not found"))?;

    if new_data.height == 0 {
        // if new_block is the first block in the chain
        // TODO: add some magic I guess
        return Ok(());
    }
    if new_data.height - 1 != prev_data.height {
        return Err(anyhow!("block height not match expected"));
    }
    if new_data.previous != Some(prev_block.get_id()?.to_vec()) {
        return Err(anyhow!("previous block did not match as expected"));
    }
    Ok(())
}
