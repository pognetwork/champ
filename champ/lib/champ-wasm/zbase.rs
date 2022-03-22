use encoding::zbase32::{FromZbase, ToZbase};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "zbase")]
pub struct Zbase();

#[wasm_bindgen(js_name = "zbase")]
impl Zbase {
    #[wasm_bindgen(catch, js_name = "decode")]
    pub fn decode(data: String) -> Result<Vec<u8>, JsError> {
        Vec::from_zbase(data).map_err(|e| JsError::new(&format!("failed to decode zbase: {}", e)))
    }

    pub fn encode(data: Vec<u8>) -> Result<String, JsError> {
        data.encode_zbase().map_err(|e| JsError::new(&format!("failed to encode as zbase: {}", e)))
    }
}
