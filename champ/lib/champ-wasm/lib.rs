use base64::encode;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Wallet {}

#[allow(clippy::unused_unit)]
#[wasm_bindgen]
impl Wallet {
    //#[wasm_bindgen(catch, js_name = "generate")]
    //pub fn generate_wallet(password: String) -> Result<(String, String), JsValue> {
    //    lulw::generate_wallet(password).map_err(|e| JsValue::from_str(&e.to_string()))
    //}
    //TODO fix this

    #[wasm_bindgen(catch, js_name = "unlock")]
    pub fn unlock_wallet(wallet: String, password: String) -> Result<String, JsValue> {
        lulw::unlock_wallet(&wallet, password).map_err(|e| JsValue::from_str(&e.to_string())).map(encode)
    }
}
