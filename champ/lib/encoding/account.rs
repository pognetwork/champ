use std::convert::TryInto;

use crypto::hash::sha3;
use thiserror::Error;

use crate::zbase32::FromZbase;

const ACCOUNT_ADDRESS_PREFIX: u8 = 0b0000_0000; // type_version

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("unknown error")]
    Unknown,
    #[error("invalid size")]
    InvalidSizeError,
    #[error("invalid checksum")]
    InvalidChecksum,
}

pub fn generate_account_address(public_key: Vec<u8>) -> Result<[u8; 24], AccountError> {
    let mut account_address = vec![ACCOUNT_ADDRESS_PREFIX];
    account_address.extend(&sha3(public_key)[0..20]);
    account_address.extend(&sha3(&account_address)[0..3]);
    account_address.try_into().map_err(|_| AccountError::InvalidSizeError)
}

pub fn parse_account_address_string(addr: &str) -> Result<Vec<u8>, AccountError> {
    let address = match addr.strip_prefix("pog-") {
        Some(a) => a,
        None => addr,
    };

    Vec::from_zbase(address).map_err(|_| AccountError::Unknown)
}

pub fn validate_account_address_string(addr: &str) -> Result<(), AccountError> {
    validate_account_address(parse_account_address_string(addr)?)
}

pub fn validate_account_address(address: Vec<u8>) -> Result<(), AccountError> {
    if address.len() != 24 {
        return Err(AccountError::InvalidSizeError);
    }

    let (address, checksum) = address.split_at(21);
    if checksum != &sha3(&address)[0..3] {
        return Err(AccountError::InvalidChecksum);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zbase32::FromZbase;

    #[test]
    fn test_generate_account_address() {
        assert_eq!(
            generate_account_address(b"test".to_vec()).unwrap().to_vec(),
            Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap()
        );
    }

    #[test]
    fn test_validate_account_address_string() {
        validate_account_address_string("pog-yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").expect("no error");
        validate_account_address_string("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").expect("no error");
        // negative tests
        assert_eq!(
            validate_account_address_string("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxg%"),
            Err(AccountError::Unknown)
        );
    }

    #[test]
    fn test_validate_account_address() {
        validate_account_address(Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgta").unwrap())
            .expect("no error");
        // negative tests
        assert_eq!(
            validate_account_address(Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgtb").unwrap()),
            Err(AccountError::InvalidChecksum)
        );
        assert_eq!(
            validate_account_address(Vec::from_zbase("yy5xyknabqan31b8fkpyrd4nydtwpausi3kxgtaa").unwrap()),
            Err(AccountError::InvalidSizeError)
        );
    }
}
