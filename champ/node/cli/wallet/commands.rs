use crate::{cli::error::CLIError, state::ChampStateArc};

use clap::ArgMatches;
use tracing::{debug, log};

pub async fn run(matches: &ArgMatches, state: &ChampStateArc) -> Result<(), CLIError> {
    debug!("check cli arguments");
    if let Some(matches) = matches.subcommand_matches("generate") {
        debug!("attempting to generate a new wallet");

        let password =
            matches.value_of("password").ok_or_else(|| CLIError::Unknown("missing password".to_string()))?;

        pwned::pwned_check(password).await.map_err(|e| CLIError::Unknown(format!("password security error: {e}")))?;

        let wallet = {
            let mut wallet_manager = state.wallet_manager.write().await;
            wallet_manager.create_wallet(password).await
        }
        .map_err(|e| CLIError::Unknown(format!("failed to generate wallet: {e}")))?;

        if matches.is_present("primary") {
            let mut config = state.config.write().await;
            config.consensus.primary_wallet = Some(wallet.clone());
            config.write().map_err(|e| CLIError::Unknown(format!("failed to update primary wallet: {e}")))?
        }

        log::info!("Successfully created wallet: {wallet}");
        return Ok(());
    }

    Err(CLIError::UnknownCommand)
}
