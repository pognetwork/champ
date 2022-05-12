use crypto::hash::sha3;
use encoding::account::{generate_account_address, validate_account_address_string};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "util")]
pub struct Util();

pub type TransactionID = [u8; 32];

pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
    [first, second].concat()
}

#[wasm_bindgen(js_name = "util")]
impl Util {
    #[wasm_bindgen(catch, js_name = "getTransactionID")]
    pub fn get_transaction_id(parent_block_id: Vec<u8>, index: u32) -> Result<Vec<u8>, JsError> {
        let parent_block_id: TransactionID = parent_block_id.try_into().map_err(|_| JsError::new("invalid tx id"))?;

        match parent_block_id.is_empty() {
            true => Err(JsError::new("parent_block_id cannot be empty")),
            false => Ok(sha3(concat_u8(&parent_block_id, &index.to_be_bytes())).to_vec()),
        }
    }

    #[wasm_bindgen(catch, js_name = "getBlockID")]
    pub fn get_block_id(data_raw: Vec<u8>, public_key: Vec<u8>) -> Vec<u8> {
        sha3(concat_u8(&data_raw, &public_key)).to_vec()
    }

    pub fn account_id_from_public_key(public_key: Vec<u8>) -> Result<Vec<u8>, JsError> {
        generate_account_address(public_key).map_err(|_| JsError::new("invalid public key")).map(|x| x.to_vec())
    }

    #[wasm_bindgen(js_name = "validateAddress")]
    pub fn validate_address(addr: &str) -> bool {
        validate_account_address_string(addr).is_ok()
    }

    #[wasm_bindgen(js_name = "sha3")]
    pub fn sha3(data: Vec<u8>) -> Vec<u8> {
        sha3(data).to_vec()
    }

    #[wasm_bindgen(js_name = "u32ToBeBytes")]
    pub fn u32_to_be_bytes(number: u32) -> Vec<u8> {
        number.to_be_bytes().to_vec()
    }
}
