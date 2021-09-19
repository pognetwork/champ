use core::iter;
use thiserror::Error;

const ZBASE_CHARS_LOWER: &[u8; 32] = b"ybndrfg8ejkmcpqxot1uwisza345h769";
const ZBASE_CHARS_UPPER: &[u8; 32] = b"YBNDRFG8EJKMCPQXOT1UWISZA345H769";

#[derive(Error, Debug)]
pub enum ZbaseError {
    #[error("unknown error")]
    Unknown,
}

pub trait ToZbase {
    fn encode_zbase<T: iter::FromIterator<char>>(&self) -> T;
    fn encode_zbase_upper<T: iter::FromIterator<char>>(&self) -> T;
}

impl<T: AsRef<[u8]>> ToZbase for T {
    fn encode_zbase<U: iter::FromIterator<char>>(&self) -> U {
        unimplemented!()
    }

    fn encode_zbase_upper<U: iter::FromIterator<char>>(&self) -> U {
        unimplemented!()
    }
}

pub trait FromZbase: Sized {
    fn from_zbase<T: AsRef<[u8]>>(hex: T) -> Result<Self, ZbaseError>;
}

impl FromZbase for Vec<u8> {
    fn from_zbase<T: AsRef<[u8]>>(hex: T) -> Result<Self, ZbaseError> {
        unimplemented!()
    }
}

pub fn encode() {}
pub fn decode() {}
