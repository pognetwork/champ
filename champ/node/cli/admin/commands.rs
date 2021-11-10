use crate::{
    cli::{admin::create_user, error::CLIError},
    state::ChampStateArc,
};

use clap::ArgMatches;
use crypto::signatures::ecdsa;
use tracing::{debug, trace};

pub async fn run(matches: &ArgMatches, state: &ChampStateArc) -> Result<(), CLIError> {
    debug!("check cli arguments");
    if let Some(matches) = matches.subcommand_matches("create-user") {
        debug!("attempting to create a user");
        {
            let config = state.config.read().await;

            if config.admin.jwt_private_key.is_empty() || config.admin.jwt_public_key.is_empty() {
                return Err(CLIError::NoKeyPair);
            }
        }

        let user = matches.value_of("username").ok_or_else(|| CLIError::Unknown("username missing".to_string()))?;
        let password =
            matches.value_of("password").ok_or_else(|| CLIError::Unknown("password missing".to_string()))?;

        pwned::pwned_check(password)
            .await
            .map_err(|e| CLIError::Unknown("pwned error: ".to_string() + &e.to_string()))?;

        let permissions = if let Some(permissions) = matches.values_of("perms") {
            permissions.map(|s| s.to_string()).collect()
        } else {
            vec![]
        };

        create_user::run(state, user, password, permissions).await?;

        trace!("Successfully created user {}", user);

        return Ok(());
    }
    if matches.subcommand_matches("generate-key").is_some() {
        let mut config = state.config.write().await;

        let key_pair =
            ecdsa::generate_key_pair().map_err(|_| CLIError::Unknown("could not generate keypair".to_string()))?;
        config.admin.jwt_public_key = key_pair.public_key;
        config.admin.jwt_private_key = key_pair.private_key;
        config.write().map_err(|e| CLIError::Unknown(e.to_string()))?;
        debug!("Successfully generated JWT keys");
        return Ok(());
    }

    Err(CLIError::UnknownCommand)
}
