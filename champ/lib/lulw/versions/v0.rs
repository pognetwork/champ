use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Lulw<'a> {
    #[serde(rename = "$schema")]
    pub _schema: &'a str,
    #[serde(rename = "$id")]
    pub _id: &'a str,

    pub version: u8,
    #[serde(borrow)]
    pub crypto: CryptoOptions<'a>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Cipher {
    #[serde(rename = "chacha20-poly1305-aead")]
    Chacha20Poly1305AEAD,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum KDF {
    #[serde(rename = "argon2id")]
    Argon2ID,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct CryptoOptions<'a> {
    pub cipherparams: CipherParams<'a>,
    pub ciphertext: &'a str,
    pub cipher: Cipher,
    pub kdf: KDF,
    pub kdfparams: KDFParams<'a>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct CipherParams<'a> {
    pub nonce: &'a str,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct KDFParams<'a> {
    pub salt: &'a str,
    pub v: u16,
    pub m: u16,
    pub y: u16,
    pub p: u16,
}
