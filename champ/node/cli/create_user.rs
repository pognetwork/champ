use crate::{cli::error::CLIError, state::ChampStateMutex};
use crypto::password::{generate_salt, hash};

pub async fn run(_state: &ChampStateMutex, _user: &str, password: &str) -> Result<(), CLIError> {
    // generate new salt
    let salt = generate_salt().map_err(|_| CLIError::Salt)?;
    // create hash
    let hash = hash(password.as_bytes(), &salt)
        .map_err(|e| CLIError::Unknown("Password hash error: ".to_string() + &e.to_string()))?;
    // print hash -> for now
    println!("{}", hash);
    Ok(())
}
