#![allow(clippy::unused_unit)]

use encoding::account::validate_account_address_string;
use wasm_bindgen::prelude::*;

pub mod adad;
pub mod wallet;
pub mod zbase;

#[wasm_bindgen(js_name = "validateAddress")]
pub fn validate_address(addr: &str) -> bool {
    validate_account_address_string(addr).is_ok()
}
