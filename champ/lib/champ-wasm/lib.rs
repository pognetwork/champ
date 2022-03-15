#![allow(clippy::unused_unit)]
use base64::decode;
use encoding::zbase32::ToZbase;
use wasm_bindgen::prelude::*;
use zeroize::Zeroize;

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum WalletKind {
    JSON,
    PrivateKey,
}

#[wasm_bindgen]
pub struct Wallet {
    json: Option<String>,
    address: Option<String>,
    private_key: Option<Vec<u8>>,
    kind: WalletKind,
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(getter = json)]
    pub fn json(&self) -> Option<String> {
        self.json.clone()
    }

    #[wasm_bindgen(getter = address)]
    pub fn address(&self) -> Option<String> {
        self.address.clone()
    }

    #[wasm_bindgen(getter = kind)]
    pub fn kind(&self) -> WalletKind {
        self.kind
    }

    #[wasm_bindgen(getter = locked)]
    pub fn locked(&self) -> bool {
        self.private_key.is_some()
    }
}

#[wasm_bindgen(catch, js_name = "generateJSONWallet")]
pub fn generate_wallet(password: &str) -> Result<Wallet, JsError> {
    let (wallet_json, wallet_address) = lulw::generate_wallet(password).map_err(|e| JsError::new(&e.to_string()))?;

    Ok(Wallet {
        json: Some(wallet_json),
        address: Some(wallet_address),
        kind: WalletKind::JSON,
        private_key: None,
    })
}

fn account_address_from_key(private_key: &[u8]) -> Result<String, JsError> {
    let public_key = crypto::signatures::ed25519::create_public_key(private_key)
        .map_err(|_| JsError::new("invalid private key"))?;
    let addr = encoding::account::generate_account_address(public_key.to_vec())
        .map_err(|_| JsError::new("failed to generate account address"))?;

    addr.encode_zbase().map_err(|_| JsError::new("failed to encode account address"))
}

#[wasm_bindgen(js_name = "walletFromJson")]
pub fn from_json(json: String) -> Wallet {
    Wallet {
        json: Some(json),
        address: None,
        kind: WalletKind::JSON,
        private_key: None,
    }
}

#[wasm_bindgen(js_name = "adadEncodeProto")]
pub fn adad_encode_proto(associated_data: &[u8], authenticated_data: &[u8]) -> Vec<u8> {
    encoding::adad::default.encode(encoding::adad::Data {
        associated_data: associated_data.to_vec(),
        authenticated_data: authenticated_data.to_vec(),
        associated_data_codec: encoding::adad::Codecs::Protobuf as usize,
        authenticated_data_codec: encoding::adad::Codecs::Protobuf as usize,
    })
}

#[wasm_bindgen(catch, js_name = "walletFromPrivateKey")]
pub fn from_private_key(key: String) -> Result<Wallet, JsError> {
    Ok(Wallet {
        json: None,
        address: None,
        kind: WalletKind::PrivateKey,
        private_key: Some(decode(key)?),
    })
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(catch, js_name = "unlock")]
    pub fn unlock(&mut self, password: &str) -> Result<(), JsError> {
        if self.private_key.is_some() {
            return Err(JsError::new("wallet is already unlocked"));
        }

        if self.kind != WalletKind::JSON {
            return Err(JsError::new("wallet can't be unlocked"));
        }

        let json = self.json.clone().ok_or_else(|| JsError::new("no json wallet"))?;
        let private_key = lulw::unlock_wallet(&json, password).map_err(|e| JsError::new(&e.to_string()))?;
        self.address = Some(account_address_from_key(&private_key)?);
        self.private_key = Some(private_key);
        Ok(())
    }

    #[wasm_bindgen(js_name = "lock")]
    pub fn lock(&mut self) {
        self.private_key.zeroize();
        self.private_key = None;
    }

    #[wasm_bindgen(catch, js_name = "sign")]
    pub fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        if let Some(private_key) = &self.private_key {
            crypto::signatures::ed25519::create_signature(data, private_key)
                .map_err(|e| JsError::new(&format!("failed to sign data: {e}")))
                .map(|c| c.to_vec())
        } else {
            Err(JsError::new("wallet is locked"))
        }
    }
}
