use crate::{
    cli::{admin::create_user, error::CLIError},
    state::ChampStateArc,
};

use clap::ArgMatches;
use crypto::curves::ecdsa;

pub async fn run(matches: &ArgMatches, state: &ChampStateArc) -> Result<(), CLIError> {
    if let Some(matches) = matches.subcommand_matches("create-user") {
        {
            let config = state.config.read().await;

            if config.admin.jwt_private_key.is_empty() || config.admin.jwt_public_key.is_empty() {
                return Err(CLIError::NoKeyPair);
            }
        }

        //TODO: Add length checks
        let user = matches.value_of("username").ok_or_else(|| CLIError::Unknown("no user name given".to_string()))?;
        let password =
            matches.value_of("password").ok_or_else(|| CLIError::Unknown("no password given".to_string()))?;
        create_user::run(state, user, password).await?;
        println!("Successfully created user '{}'", user);

        return Ok(());
    }
    if matches.subcommand_matches("generate-key").is_some() {
        let mut config = state.config.write().await;

        let key_pair =
            ecdsa::generate_key_pair().map_err(|_| CLIError::Unknown("could not generate keypair".to_string()))?;
        config.admin.jwt_public_key = key_pair.public_key;
        config.admin.jwt_private_key = key_pair.private_key;
        config.write().map_err(|e| CLIError::Unknown(e.to_string()))?;
        println!("Successfully generated JWT keys");
        return Ok(());
    }

    Err(CLIError::UnknownCommand)
}
