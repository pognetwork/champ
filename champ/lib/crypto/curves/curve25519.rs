use anyhow::Result;
use ring::{
    rand,
    signature::{self, KeyPair},
};
// https://www.youtube.com/watch?v=NF1pwjL9-DE Computerphile Vid
// increment nr of times watched without understanding: 3

/// Verifiy the signature of some data
pub fn verify_signature(data: &Vec<u8>, public_key: &Vec<u8>, data_signature: &Vec<u8>) -> Result<()> {
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    peer_public_key.verify(data, data_signature)?;
    Ok(())
}

/// create signature from data
pub fn create_signature(data: &Vec<u8>, private_key: &Vec<u8>) -> Result<Vec<u8>> {
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key)?;
    Ok(key_pair.sign(data).as_ref().to_vec())
}

/// create new private key
pub fn generate_private_key() -> Result<Vec<u8>> {
    let rng = rand::SystemRandom::new();
    let key = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    Ok(key.as_ref().to_vec())
}

/// create a key pair from a private key
pub fn create_public_key(private_key: &Vec<u8>) -> Result<Vec<u8>> {
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key)?;
    Ok(key_pair.public_key().as_ref().to_vec())
}
