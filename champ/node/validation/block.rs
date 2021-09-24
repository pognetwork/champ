use anyhow::{anyhow, Result};
use crypto::{self, curves::curve25519::verify_signature};
use pog_proto::api::Block;
use prost::Message;

// Validate block
pub fn validate(block: &Block) -> Result<()> {
    let data = block
        .clone()
        .data
        .ok_or_else(|| anyhow!("block data not found"))?
        .encode_to_vec();
    let public_key = &block.public_key;
    let signature = &block.signature;

    // signature
    verify_signature(&data, public_key, signature)?;
    // transactions / balance
    // height / previous block

    Ok(())
}

// Verifies the transactions and balances
fn verify_transactions() -> Result<()> {
    unimplemented!()
}

// Verifies the block height and previous block
fn verify_previous_block() -> Result<()> {
    unimplemented!()
}
