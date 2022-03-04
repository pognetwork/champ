mod versions;
use std::convert::TryInto;

use base64::{decode, encode};
use crypto::aead::chacha::{decrypt, encrypt};
use encoding::account::generate_account_address;
use encoding::zbase32::ToZbase;
use thiserror::Error;
use tracing::debug;
use versions::v0::{Cipher, CipherParams, CryptoOptions, KDFParams, Lulw, KDF};

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("error calculating public key: {0}")]
    CreatePublicKeyError(String),
    #[error("error generating public key: {0}")]
    GeneratePrivateKeyError(String),
    #[error("error encrypting private key: {0}")]
    EncryptionError(String),
    #[error("error encrypting private key: {0}")]
    EncodingError(String),

    #[error(transparent)]
    AccountError(#[from] encoding::account::AccountError),

    #[error(transparent)]
    ZbaseError(#[from] encoding::zbase32::ZbaseError),

    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),

    #[error("failed to decode base64: {0}")]
    DecodeError(#[from] base64::DecodeError),

    #[error("Invalid property: {0}")]
    InvalidProperty(String),

    #[error("invalid wallet format")]
    InvalidWalletFormat,

    #[error("invalid wallet kdf params")]
    InvalidKDFParams,

    #[error("error decrypting private key: {0}")]
    DecryptionError(String),
}

pub type WalletAndAddress = (String, String);

pub fn generate_wallet(password: String) -> Result<WalletAndAddress, WalletError> {
    debug!("generating wallet");
    let (public_key, (ciphertext, salt, nonce)) = {
        let private_key = crypto::signatures::ed25519::generate_private_key()
            .map_err(|e| WalletError::GeneratePrivateKeyError(e.to_string()))?;

        let public_key = crypto::signatures::ed25519::create_public_key(&private_key)
            .map_err(|e| WalletError::CreatePublicKeyError(e.to_string()))?;
        (
            public_key,
            encrypt(&private_key, password.as_bytes()).map_err(|e| WalletError::EncryptionError(e.to_string()))?,
        )
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
            cipher: Cipher::XChacha20Poly1305AEAD,
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

    let json = serde_json::to_string_pretty(&wallet)?;
    let account_address = generate_account_address(public_key.to_vec())?.encode_zbase()?;

    Ok((json, account_address))
}

pub fn unlock_wallet(wallet: &str, password: String) -> Result<Vec<u8>, WalletError> {
    let parsed_wallet: Lulw = serde_json::from_str(wallet).map_err(WalletError::SerializationError)?;

    if !(parsed_wallet.version == 0
        && parsed_wallet.crypto.cipher == Cipher::XChacha20Poly1305AEAD
        && parsed_wallet.crypto.kdf == KDF::Argon2ID)
    {
        return Err(WalletError::InvalidWalletFormat);
    }

    if !(parsed_wallet.crypto.kdfparams.v == 19
        && parsed_wallet.crypto.kdfparams.m == 4096
        && parsed_wallet.crypto.kdfparams.y == 3
        && parsed_wallet.crypto.kdfparams.p == 1)
    {
        return Err(WalletError::InvalidKDFParams);
    }

    let data = decode(parsed_wallet.crypto.ciphertext).map_err(WalletError::DecodeError)?;

    let salt: [u8; 16] = decode(parsed_wallet.crypto.kdfparams.salt)
        .map_err(WalletError::DecodeError)?
        .try_into()
        .map_err(|_| WalletError::InvalidProperty("salt".to_string()))?;

    let nonce: [u8; 24] = decode(parsed_wallet.crypto.cipherparams.nonce)
        .map_err(WalletError::DecodeError)?
        .try_into()
        .map_err(|_| WalletError::InvalidProperty("salt".to_string()))?;

    let private_key =
        decrypt(&data, password.as_bytes(), salt, nonce).map_err(|e| WalletError::DecryptionError(e.to_string()))?;

    Ok(private_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlock_wallet_test() {
        let wallet = "{
            \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",
            \"$id\": \"https://schemas.pog.network/lulw.schema.json\",
            \"version\": 0,
            \"crypto\": {
                \"cipherparams\": {
                    \"nonce\": \"ECWqXqXq4N2/efQjwXmaxFYu9P4ooDTa\"
                },
                \"ciphertext\": \"AAekhDj16MuQCu9LlZNgS6rAQ6ed5/w0/gFfwdSd9FKVStHdYSwlYBmBCVd4ZY7W\",
                \"cipher\": \"xchacha20-poly1305-aead\",
                \"kdf\": \"argon2id\",
                \"kdfparams\": {
                    \"salt\": \"CZsaQKR9EBwE3jHJE9SJsA==\",
                    \"v\": 19,
                    \"m\": 4096,
                    \"y\": 3,
                    \"p\": 1
                }
            }
        }";
        let passphrase = unlock_wallet(wallet, "1234".to_string());
        assert!(passphrase.is_ok())
    }

    #[test]
    fn create_wallet_which_is_unlockable() {
        let password = "1234".to_string();
        let (wallet, _) = generate_wallet(password.clone()).expect("Couldn't generate wallet");

        let result = unlock_wallet(wallet.as_str(), password);
        assert!(result.is_ok())
    }
}
