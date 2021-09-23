use thiserror::Error;

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};

use ring::aead::{self, Aad, LessSafeKey};
use ring::rand::{SecureRandom, SystemRandom};

#[derive(Error, Debug)]
pub enum AeadError {
    #[error("unknown error")]
    Unknown,
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
type Nonce = [u8; 12];

pub fn encrypt(data: &[u8], password: &[u8]) -> Result<(Vec<u8>, Salt, Nonce), AeadError> {
    let rand = SystemRandom::new();

    let mut nonce: [u8; 12] = [0; 12];
    let mut salt: [u8; 16] = [0; 16];

    rand.fill(&mut nonce).map_err(|_| AeadError::RandomFillError)?;
    rand.fill(&mut salt).map_err(|_| AeadError::RandomFillError)?;

    let key = get_ring_key(&hash_key(password, &salt)?)?;
    let mut in_out = data.to_owned();

    key.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut in_out)
        .map_err(|_| AeadError::Unknown)?;

    Ok((in_out, salt, nonce))
}

fn hash_key(key: &[u8], salt: &[u8]) -> Result<Vec<u8>, AeadError> {
    let salt = SaltString::b64_encode(salt).map_err(|_| AeadError::SaltEncodingError)?;

    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(key, &salt)
        .map_err(|_| AeadError::PasswordHashError)?
        .hash
        .ok_or(AeadError::PasswordHashError)?
        .as_bytes()
        .to_owned())
}

fn get_ring_key(key: &[u8]) -> Result<LessSafeKey, AeadError> {
    let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key).map_err(|_| AeadError::Unknown)?;
    Ok(aead::LessSafeKey::new(unbound_key))
}

pub fn decrypt(data: &[u8], password: &[u8], salt: Salt, nonce: Nonce) -> Result<Vec<u8>, AeadError> {
    let key = get_ring_key(&hash_key(password, &salt)?)?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce);

    let total_len = data.len() + key.algorithm().tag_len();
    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(data);

    let decrypted = key
        .open_in_place(nonce, Aad::empty(), &mut buf)
        .map_err(|_| AeadError::OpenError)?;

    Ok(decrypted.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = &[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        let password = b"hunter2";
        let (encrypted_data, salt, nonce) = encrypt(data, password).unwrap();
        let unencrypted_data = decrypt(&encrypted_data, password, salt, nonce).unwrap();
        assert_eq!(data.to_vec(), unencrypted_data);
    }

    #[test]
    fn unique_password() {
        let data = &[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        let password = b"hunter2";
        let (encrypted_data, salt, nonce) = encrypt(data, password).unwrap();
        let decryption_err = decrypt(&encrypted_data, b"hunter3", salt, nonce).is_err();
        assert_eq!(decryption_err, true);
    }
}
