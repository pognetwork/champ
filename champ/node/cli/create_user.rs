use crate::{cli::error::CLIError, state::ChampStateMutex};
use crypto::password::{generate_salt, hash};

pub async fn run(_state: &ChampStateMutex, _user: &str, password: &str) -> Result<(), CLIError> {
    let salt = generate_salt().map_err(|_| CLIError::Salt)?;

    let hash = hash(password.as_bytes(), &salt)
        .map_err(|e| CLIError::Unknown("Password hash error: ".to_string() + &e.to_string()))?;

    println!("{}", hash);
    Ok(())
}
