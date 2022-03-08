#![allow(clippy::unused_unit)]

use base64::encode;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Wallet {}

#[wasm_bindgen]
pub struct GeneratedWallet {
    wallet_json: String,
    wallet_address: String,
}

#[wasm_bindgen]
impl GeneratedWallet {
    #[wasm_bindgen(getter = json)]
    pub fn wallet_json(&self) -> String {
        self.wallet_json.clone()
    }

    #[wasm_bindgen(getter = address)]
    pub fn wallet_address(&self) -> String {
        self.wallet_address.clone()
    }
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(catch, js_name = "generate")]
    pub fn generate_wallet(password: &str) -> Result<GeneratedWallet, JsValue> {
        let (wallet_json, wallet_address) =
            lulw::generate_wallet(password).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(GeneratedWallet {
            wallet_address,
            wallet_json,
        })
    }

    #[wasm_bindgen(catch, js_name = "unlock")]
    pub fn unlock_wallet(wallet: &str, password: &str) -> Result<String, JsValue> {
        lulw::unlock_wallet(wallet, password).map_err(|e| JsValue::from_str(&e.to_string())).map(encode)
    }
}
