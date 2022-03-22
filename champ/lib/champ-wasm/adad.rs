use encoding::adad;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ADADData {
    associated_data: Vec<u8>,

    #[wasm_bindgen(js_name = "associatedDataCodec")]
    pub associated_data_codec: usize,

    authenticated_data: Vec<u8>,

    #[wasm_bindgen(js_name = "authenticatedDataCodec")]
    pub authenticated_data_codec: usize,
}

#[wasm_bindgen]
impl ADADData {
    #[wasm_bindgen(getter = associatedData)]
    pub fn get_associated_data(&self) -> Vec<u8> {
        self.associated_data.clone()
    }

    #[wasm_bindgen(getter = authenticatedData)]
    pub fn get_authenticated_data(&self) -> Vec<u8> {
        self.authenticated_data.clone()
    }
}

#[wasm_bindgen(js_name = "adad")]
pub struct ADAD();

#[wasm_bindgen(js_name = "adad")]
impl ADAD {
    #[wasm_bindgen(catch, js_name = "decode")]
    pub fn decode(data: &[u8]) -> Result<ADADData, JsError> {
        let data = adad::default.read(data)?;

        Ok(ADADData {
            associated_data: data.associated_data,
            associated_data_codec: data.associated_data_codec,
            authenticated_data: data.authenticated_data,
            authenticated_data_codec: data.authenticated_data_codec,
        })
    }

    pub fn encode(data: ADADData) -> Vec<u8> {
        let data = adad::Data {
            associated_data: data.associated_data,
            associated_data_codec: data.associated_data_codec,
            authenticated_data: data.authenticated_data,
            authenticated_data_codec: data.authenticated_data_codec,
        };

        adad::default.encode(data)
    }
}
