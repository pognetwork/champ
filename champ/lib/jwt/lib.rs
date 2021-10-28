use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JWTError {
    #[error("{0}")]
    Unknown(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iat: u64,
    exp: u64,
    iss: String,
    pub permissions: Vec<String>,
}

fn get_current_time() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("SystemTime before UNIX EPOCH").as_secs()
}

pub fn create(user_id: &str, expires_in_seconds: u64, private_key: &[u8]) -> Result<String, JWTError> {
    let header = Header::new(Algorithm::ES256);

    let now = get_current_time();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + expires_in_seconds,
        iss: "pog.network node".to_string(),
        permissions: vec![],
    };

    let encoding_key = &EncodingKey::from_ec_pem(private_key).map_err(|e| JWTError::Unknown(e.to_string()))?;
    let token = jsonwebtoken::encode(&header, &claims, encoding_key).map_err(|e| JWTError::Unknown(e.to_string()))?;

    Ok(token.as_str().to_string())
}

pub fn verify(token: &str, public_key: &[u8]) -> Result<Claims, JWTError> {
    let validation = Validation::new(Algorithm::ES256);

    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_ec_pem(public_key).map_err(|e| JWTError::Unknown(e.to_string()))?,
        &validation,
    )
    .map_err(|e| JWTError::Unknown(e.to_string()))?;

    Ok(claims.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLIC_KEY: &[u8] = b"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEw7JAoU/gJbZJvV+zCOvU9yFJq0FN
C/edCMRM78P8eQTBCDUTK1ywSYaszvQZvneiW6gNtWEJndSreEcyyUdVvg==
-----END PUBLIC KEY-----";

    const PRIVATE_KEY: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgWTFfCGljY6aw3Hrt
kHmPRiazukxPLb6ilpRAewjW8nihRANCAATDskChT+Altkm9X7MI69T3IUmrQU0L
950IxEzvw/x5BMEINRMrXLBJhqzO9Bm+d6JbqA21YQmd1Kt4RzLJR1W+
-----END PRIVATE KEY-----";

    #[test]
    fn it_works() {
        let jwt = create("123", 60 * 60 * 24, PRIVATE_KEY).expect("should create a jwt");
        verify(&jwt, PUBLIC_KEY).expect("should verify jwt");
    }
}
