mod versions;
use base64::encode;
use crypto::aead::encrypt;
use thiserror::Error;
use versions::v0::{Cipher, CipherParams, CryptoOptions, KDFParams, Lulw, KDF};

#[derive(Error, Debug)]
pub enum WalletError<'a> {
    #[error("error calculating public key: {0}")]
    CreatePublicKeyError(String),
    #[error("error generating private key: {0}")]
    GeneratePrivateKeyError(String),
    #[error("error encrypting private key: {0}")]
    EncryptionError(String),
    #[error("error encrypting private key: {0}")]
    EncodingError(&'a str),
    #[error("failed to serialize wallet: {0}")]
    SerializationError(String),
}

pub fn generate_wallet(password: &str) -> Result<String, WalletError> {
    let (ciphertext, salt, nonce) = {
        let private_key: Vec<u8> = crypto::curves::curve25519::generate_private_key()
            .map_err(|e| WalletError::GeneratePrivateKeyError(e.to_string()))?;

        encrypt(&private_key, password.as_bytes()).map_err(|e| WalletError::EncryptionError(e.to_string()))?
    };

    let ciphertext = encode(ciphertext);
    let salt = encode(salt);
    let nonce = encode(nonce);

    let wallet = Lulw {
        _id: "https://schemas.pog.network/lulw.schema.json",
        _schema: "https://json-schema.org/draft/2020-12/schema",
        version: 0,
        crypto: CryptoOptions {
            cipherparams: CipherParams {
                nonce: nonce.as_str(),
            },
            ciphertext: ciphertext.as_str(),
            cipher: Cipher::Chacha20Poly1305AEAD,
            kdf: KDF::Argon2ID,
            kdfparams: KDFParams {
                salt: salt.as_str(),
                v: 19,
                m: 4096,
                y: 3,
                p: 1,
            },
        },
    };

    let json = serde_json::to_string_pretty(&wallet).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    Ok(json.to_string())
}

pub fn unlock_wallet() {}
