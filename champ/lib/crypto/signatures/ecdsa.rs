use p256::SecretKey;
use pkcs8::{EncodePrivateKey, EncodePublicKey};
use rand::thread_rng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ECDSAError {
    #[error("Error creating key pair")]
    KeyPairError,
}

pub struct PEMKeyPair {
    pub public_key: String,
    pub private_key: String,
}

pub fn generate_key_pair() -> Result<PEMKeyPair, ECDSAError> {
    let private_key = SecretKey::random(thread_rng());

    let public_key =
        private_key.public_key().to_public_key_pem(pkcs8::LineEnding::CRLF).map_err(|_| ECDSAError::KeyPairError)?;
    let private_key = private_key.to_pkcs8_pem(pkcs8::LineEnding::CRLF).map_err(|_| ECDSAError::KeyPairError)?;

    Ok(PEMKeyPair {
        private_key: private_key.to_string(),
        public_key,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let _ = generate_key_pair().expect("good key generation");
    }
}
