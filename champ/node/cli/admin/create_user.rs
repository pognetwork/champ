use crate::{cli::error::CLIError, state::ChampStateArc};
use crypto::id::generate;
use crypto::password::{generate_salt, hash};

pub async fn run(state: &ChampStateArc, user: &str, password: &str, perms: Vec<String>) -> Result<(), CLIError> {
    let salt = generate_salt().map_err(|_| CLIError::Salt)?;

    let hash = hash(password.as_bytes(), &salt)
        .map_err(|e| CLIError::Unknown("Password hash error: ".to_string() + &e.to_string()))?;

    // check username exists
    let mut config = state.config.write().await;

    if config.node_users.contains_key(&user.to_string()) {
        return Err(CLIError::UserExists);
    }

    config.node_users.insert(
        user.to_string(),
        crate::config::UserAccount {
            user_id: generate().expect("insufficient randomness"),
            password_hash: hash,
            permissions: perms,
        },
    );

    config.write().map_err(|e| CLIError::Unknown(e.to_string()))?;
    Ok(())
}
