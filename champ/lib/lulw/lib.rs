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

    let json = serde_json::to_string_pretty(&wallet).map_err(|e| WalletError::SerializationError(e.to_string()))?;
    let account_address = generate_account_address(public_key.to_vec())
        .expect("Valid public key couldn't generate account address")
        .encode_zbase()
        .expect("Valid account address couldn't be converted to ZBase");
    Ok((json, account_address))
}

pub fn unlock_wallet(wallet: &str, password: String) -> Result<Vec<u8>, WalletError> {
    let wallet: Lulw = serde_json::from_str(wallet).map_err(|e| WalletError::DeserializationError(e.to_string()))?;

    if !(wallet.version == 1
        && wallet.crypto.cipher == Cipher::XChacha20Poly1305AEAD
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

    let nonce: [u8; 24] = decode(wallet.crypto.cipherparams.nonce)
        .map_err(|_| WalletError::DecodeError("nonce".to_string()))?
        .try_into()
        .map_err(|_| WalletError::DecodeError("nonce".to_string()))?;

    let private_key =
        decrypt(&data, password.as_bytes(), salt, nonce).map_err(|e| WalletError::DecryptionError(e.to_string()))?;

    Ok(private_key)
}
