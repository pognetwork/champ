use crypto::hash::create_sha1_password;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PwnedError {
    #[error("{0}")]
    Internal(String),
    #[error("Passwords Been Pwned")]
    Pwned,
}

pub async fn pwned_check(password: &str) -> Result<(), PwnedError> {
    let pw_hash = create_sha1_password(password);
    let pw_str = hex::encode(&pw_hash).to_uppercase();
    let (prefix, suffex) = pw_str.split_at(5);
    let response = reqwest::get("https://api.pwnedpasswords.com/range/".to_owned() + prefix)
        .await
        .map_err(|e| PwnedError::Internal("reqwest failed: ".to_string() + &e.to_string()))?;
    let response_body =
        response.text().await.map_err(|e| PwnedError::Internal("text error: ".to_string() + &e.to_string()))?;

    if response_body.split('\n').any(|row| row.starts_with(suffex)) {
        return Err(PwnedError::Pwned);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::pwned_check;
    use crate::PwnedError;

    #[tokio::test]
    async fn test_pwned_check() -> Result<(), PwnedError> {
        assert_eq!(Err(PwnedError::Pwned), pwned_check("password").await);
        assert_eq!(Err(PwnedError::Pwned), pwned_check("adminadmin").await);
        assert_eq!(Ok(()), pwned_check("flkatoihkvdjnasdjölewm").await);
        Ok(())
    }
}
