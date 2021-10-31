use ring::{
    rand,
    signature::{self, KeyPair},
};

pub use ring::signature::Ed25519KeyPair;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Ed25519Error {
    #[error("unknown error")]
    Unknown,
    #[error("Unable to verify signature")]
    VerificationError,
    #[error("Unable to create signature")]
    CreateSignatureError,
    #[error("Unable to create keypair")]
    KeyPairError,
}

// https://www.youtube.com/watch?v=NF1pwjL9-DE Computerphile Vid
// increment nr of times watched without understanding: 3

//A                  B
//APrivate           BPrivate
//BPublic            APublic

// A Data ---------------------> B
// BPublic(Data) --------------> B BPrivate = Data
// APrivate(Data)--------------> B APublic = Data
// APrivate(BPublic(Data)) ----> B

// At B
// APrivate(BPublic(Data)) % APublic = BPublic(Data)
// BPublic(Data) % BPrivate = Data

/// Verify the signature of some data
pub fn verify_signature(data: &[u8], public_key: &[u8], data_signature: &[u8]) -> Result<(), Ed25519Error> {
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    peer_public_key.verify(data, data_signature).map_err(|_| Ed25519Error::VerificationError)?;
    Ok(())
}

/// create signature from data
pub fn create_signature(data: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Ed25519Error> {
    let key_pair =
        signature::Ed25519KeyPair::from_pkcs8(private_key).map_err(|_| Ed25519Error::CreateSignatureError)?;
    Ok(key_pair.sign(data).as_ref().to_vec())
}

/// create new private key
pub fn generate_private_key() -> Result<Vec<u8>, Ed25519Error> {
    let rng = rand::SystemRandom::new();
    let private_key = signature::Ed25519KeyPair::generate_pkcs8(&rng).map_err(|_| Ed25519Error::KeyPairError)?;
    Ok(private_key.as_ref().to_vec())
}

/// create a key pair from a private key
pub fn create_public_key(private_key: &[u8]) -> Result<Vec<u8>, Ed25519Error> {
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key).map_err(|_| Ed25519Error::KeyPairError)?;
    Ok(key_pair.public_key().as_ref().to_vec())
}
