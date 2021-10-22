mod versions;
use std::convert::TryInto;

use base64::{decode, encode};
use crypto::aead::{decrypt, encrypt};
use thiserror::Error;
use versions::v0::{Cipher, CipherParams, CryptoOptions, KDFParams, Lulw, KDF};

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("error calculating public key: {0}")]
    CreatePublicKeyError(String),
    #[error("error generating private key: {0}")]
    GeneratePrivateKeyError(String),
    #[error("error encrypting private key: {0}")]
    EncryptionError(String),
    #[error("error encrypting private key: {0}")]
    EncodingError(String),
    #[error("failed to serialize wallet: {0}")]
    SerializationError(String),
    #[error("failed to deserialize wallet: {0}")]
    DeserializationError(String),
    #[error("failed to decode base64: {0}")]
    DecodeError(String),
    #[error("invalid wallet format")]
    InvalidWalletFormat,
    #[error("invalid wallet kdf params")]
    InvalidKDFParams,
    #[error("error decrypting private key: {0}")]
    DecryptionError(String),
}

pub fn generate_wallet(password: String) -> Result<String, WalletError> {
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
    Ok(json)
}

pub fn unlock_wallet(wallet: &str, password: String) -> Result<Vec<u8>, WalletError> {
    let wallet: Lulw = serde_json::from_str(wallet).map_err(|e| WalletError::DeserializationError(e.to_string()))?;

    if !(wallet.version == 1
        && wallet.crypto.cipher == Cipher::Chacha20Poly1305AEAD
        && wallet.crypto.kdf == KDF::Argon2ID)
    {
        return Err(WalletError::InvalidWalletFormat);
    }

    if !(wallet.crypto.kdfparams.v == 19
        && wallet.crypto.kdfparams.m == 4096
        && wallet.crypto.kdfparams.y == 3
        && wallet.crypto.kdfparams.p == 1)
    {
        return Err(WalletError::InvalidKDFParams);
    }

    let data = decode(wallet.crypto.ciphertext).map_err(|_| WalletError::DecodeError("ciphertext".to_string()))?;

    let salt: [u8; 16] = decode(wallet.crypto.kdfparams.salt)
        .map_err(|_| WalletError::DecodeError("salt".to_string()))?
        .try_into()
        .map_err(|_| WalletError::DecodeError("salt".to_string()))?;

    let nonce: [u8; 12] = decode(wallet.crypto.cipherparams.nonce)
        .map_err(|_| WalletError::DecodeError("nonce".to_string()))?
        .try_into()
        .map_err(|_| WalletError::DecodeError("nonce".to_string()))?;

    let private_key =
        decrypt(&data, password.as_bytes(), salt, nonce).map_err(|e| WalletError::DecryptionError(e.to_string()))?;

    Ok(private_key)
}
