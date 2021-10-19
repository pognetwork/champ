use base64::encode;
use ring::rand::{SecureRandom, SystemRandom};

pub fn generate() -> Result<String, String> {
    let rand = SystemRandom::new();
    let mut id: [u8; 16] = [0; 16];
    rand.fill(&mut id).map_err(|_| "error getting random data".to_string())?;
    Ok(encode(id))
}
