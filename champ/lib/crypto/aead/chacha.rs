use rand::Rng;
use thiserror::Error;

use crate::password;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{XChaCha20Poly1305, XNonce as chachaNonce};

#[derive(Error, Debug)]
pub enum AeadError {
    #[error("unknown error: {0}")]
    Unknown(String),
    #[error("error while encoding salt")]
    SaltEncodingError,
    #[error("error getting enough random data")]
    RandomFillError,
    #[error("error hashing password")]
    PasswordHashError,
    #[error("error decrypting data")]
    OpenError,
}

type Salt = [u8; 16];
type Nonce = [u8; 24];

pub fn encrypt(data: &[u8], password: &[u8]) -> Result<(Vec<u8>, Salt, Nonce), AeadError> {
    let nonce: Nonce = rand::thread_rng().gen::<Nonce>();
    let salt: Salt = rand::thread_rng().gen::<Salt>();

    let hash: [u8; 32] = password::hash_digest(password, &salt).map_err(|_| AeadError::PasswordHashError)?[..32]
        .try_into()
        .map_err(|_| AeadError::PasswordHashError)?;

    let cipher = XChaCha20Poly1305::new(&hash.into());

    let ciphertext =
        cipher.encrypt(chachaNonce::from_slice(&nonce), data).map_err(|e| AeadError::Unknown(e.to_string()))?; // NOTE: handle this error to avoid panics!
    Ok((ciphertext, salt, nonce))
}

pub fn decrypt(ciphertext: &[u8], password: &[u8], salt: Salt, nonce: Nonce) -> Result<Vec<u8>, AeadError> {
    let hash: [u8; 32] = password::hash_digest(password, &salt).map_err(|_| AeadError::PasswordHashError)?[..32]
        .try_into()
        .map_err(|_| AeadError::PasswordHashError)?;

    let cipher = XChaCha20Poly1305::new(&hash.into());
    let decrypted = cipher.decrypt(chachaNonce::from_slice(&nonce), ciphertext).map_err(|_| AeadError::OpenError)?;
    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = &[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        let password = b"hunter2";
        let (encrypted_data, salt, nonce) = encrypt(data, password).expect("should encrypt");
        let unencrypted_data = decrypt(&encrypted_data, password, salt, nonce).expect("should decrypt");
        assert_eq!(data.to_vec(), unencrypted_data);
    }

    #[test]
    fn unique_password() {
        let data = &[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        let password = b"hunter2";
        let (encrypted_data, salt, nonce) = encrypt(data, password).expect("should encrypt");
        let decryption_err = decrypt(&encrypted_data, b"hunter3", salt, nonce).is_err();
        assert_eq!(decryption_err, true);
    }
}
