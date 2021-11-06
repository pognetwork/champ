use ring::{
    rand,
    signature::{self, KeyPair},
};
use thiserror::Error;

pub use ring::signature::EcdsaKeyPair;

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
    let rng = rand::SystemRandom::new();
    let key_bytes = signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
        .map_err(|_| ECDSAError::KeyPairError)?;
    let key_pair =
        signature::EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, key_bytes.as_ref())
            .map_err(|_| ECDSAError::KeyPairError)?;
    let private_key = pem::encode(&pem::Pem {
        tag: "PRIVATE KEY".to_string(),
        contents: key_bytes.as_ref().to_vec(),
    });
    let public_key = pem::encode(&pem::Pem {
        tag: "PUBLIC KEY".to_string(),
        contents: key_pair.public_key().as_ref().to_vec(),
    });

    Ok(PEMKeyPair {
        public_key,
        private_key,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        generate_key_pair().expect("good key generation");
    }
}
