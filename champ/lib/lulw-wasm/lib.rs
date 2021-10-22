use base64::encode;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(catch, js_name = "generateWallet")]
pub fn generate_wallet(password: String) -> Result<String, JsValue> {
    lulw::generate_wallet(password).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(catch, js_name = "unlockWallet")]
pub fn unlock_wallet(wallet: String, password: String) -> Result<String, JsValue> {
    lulw::unlock_wallet(&wallet, password).map_err(|e| JsValue::from_str(&e.to_string())).map(encode)
}
