use ed25519_zebra::{SigningKey, VerificationKey};
use rand::thread_rng;
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

/// Verify the signature of some data
pub fn verify_signature(data: &[u8], public_key: &[u8], data_signature: &[u8]) -> Result<(), Ed25519Error> {
    let sk = VerificationKey::try_from(public_key).map_err(|_| Ed25519Error::VerificationError)?;
    let sig_bytes: [u8; 64] = data_signature.try_into().map_err(|_| Ed25519Error::VerificationError)?;
    sk.verify(&sig_bytes.into(), data).map_err(|_| Ed25519Error::VerificationError)?;
    Ok(())
}

// /// create signature from data
pub fn create_signature(data: &[u8], private_key: &[u8]) -> Result<[u8; 64], Ed25519Error> {
    let sk = SigningKey::try_from(private_key).map_err(|_| Ed25519Error::VerificationError)?;
    let sig_bytes: [u8; 64] = sk.sign(data).into();
    Ok(sig_bytes)
}

// /// create new private key
pub fn generate_private_key() -> Result<[u8; 32], Ed25519Error> {
    let sk = SigningKey::new(thread_rng());
    let priv_key: [u8; 32] = sk.into();
    Ok(priv_key)
}

// /// create a public key from a private key
pub fn create_public_key(private_key: &[u8]) -> Result<[u8; 32], Ed25519Error> {
    let sk = SigningKey::try_from(private_key).map_err(|_| Ed25519Error::VerificationError)?;
    let pk: [u8; 32] = VerificationKey::from(&sk).into();
    Ok(pk)
}
