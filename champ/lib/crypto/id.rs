use base64::encode;
use rand::{thread_rng, Rng};

pub fn generate() -> Result<String, String> {
    let id: [u8; 16] = thread_rng().gen();
    Ok(encode(id))
}
