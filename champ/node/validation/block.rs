use anyhow::{anyhow, Result};
use crypto;
use pog_proto::api::Block;
use prost::Message;

// Get Block hash
#[allow(dead_code)]
pub fn get_hash(block: Block) -> Result<Vec<u8>> {
    let data = block
        .data
        .ok_or_else(|| anyhow!("Block data was None"))?
        .encode_to_vec();
    Ok(crypto::hash::sha3(data))
}

// Validate block
// block signature
pub fn signature() {
    
}