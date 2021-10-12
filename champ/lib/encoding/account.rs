use std::convert::TryInto;

use crypto::hash::sha3;
use thiserror::Error;

const ACCOUNT_ADDRESS_PREFIX: u8 = 0b0000_0000; // type_version

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("unknown error")]
    Unknown,
    #[error("invalid size")]
    InvalidSizeError,
}

pub fn generate_account_address(public_key: Vec<u8>) -> Result<[u8; 24], AccountError> {
    let mut account_address = vec![ACCOUNT_ADDRESS_PREFIX];
    account_address.extend(&sha3(public_key)[0..20]);
    account_address.extend(&sha3(&account_address)[0..3]);
    account_address.try_into().map_err(|_| AccountError::InvalidSizeError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zbase32::FromZbase;

    #[test]
    fn account_address() {
        assert_eq!(
            generate_account_address(b"test".to_vec()).unwrap().to_vec(),
            Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap()
        );
    }
}
