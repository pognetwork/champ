use ring::{rand,signature};
use anyhow::Result;
// https://www.youtube.com/watch?v=NF1pwjL9-DE Computerphile Vid
// increment nr of times watched without understanding: 2

/// Verifiy the signature of some data
pub fn verify_signature(data: Vec<u8>, public_key: Vec<u8>, signature: Vec<u8>) -> Result<()> {
    let peer_pk = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    peer_pk.verify(&data, signature.as_ref())?;
    Ok(())
}

/// create signature from data
pub fn create_signature(data: Vec<u8>, private_key: Vec<u8>) -> Result<Vec<u8>> {
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key.as_ref())?;
    Ok(key_pair.sign(&data).as_ref().to_vec())
}

pub fn generate_private_key() -> Result<Vec<u8>> {
    let rng = rand::SystemRandom::new();
    let key = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    Ok(key.as_ref().to_vec())
}