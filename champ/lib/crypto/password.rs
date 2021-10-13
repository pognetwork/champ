use argon2::{
    self,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("error hashing password: {0}")]
    Hash(String),
    #[error("error verifying password")]
    Verify,
    #[error("error hashing password")]
    PwHash,
}

pub fn hash(password: &[u8], salt: &[u8]) -> Result<String, PasswordError> {
    let salt = SaltString::b64_encode(salt).map_err(|_| PasswordError::Hash("error encoding salt".to_string()))?;

    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password, &salt).map_err(|e| PasswordError::Hash(e.to_string()))?.to_string())
}

pub fn verify(password: &[u8], hash: &str) -> Result<(), PasswordError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash).expect("idk");
    Ok(argon2.verify_password(password, &parsed_hash).map_err(|_| PasswordError::Verify)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let password = b"hunter2";
        let salt = crate::hash::sha3(b":)");

        let some_hash = hash(password, &salt).expect("should hash");
        verify(password, &some_hash).expect("should be the same");
    }
}
