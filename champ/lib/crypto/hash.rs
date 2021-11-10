use sha3::{Digest, Sha3_256};

pub fn sha3(data: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn create_sha1_password(password: &str) -> [u8; 20] {
    let mut hasher = sha1::Sha1::new();
    hasher.update(password);
    hasher.finalize().into()
}
