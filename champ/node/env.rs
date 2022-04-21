use anyhow::{anyhow, Result};
use std::env;

use crate::{
    cli::admin::{create_user, generate_jwt_keys},
    state::ChampStateArc,
};

static CHAMP_PRIMARY_WALLET_PASSWORD: &str = "CHAMP_PRIMARY_WALLET_PASSWORD";
static CHAMP_INITIAL_PEERS: &str = "CHAMP_INITIAL_PEERS";
static CHAMP_GENERATE_PRIMARY_WALLET: &str = "CHAMP_GENERATE_PRIMARY_WALLET";
static CHAMP_GENERATE_JWT_KEYS: &str = "CHAMP_GENERATE_JWT_KEYS";
static CHAMP_DEBUG_CREATE_SUPERADMIN: &str = "CHAMP_DEBUG_CREATE_SUPERADMIN";

/// process_env processes champ-related environment variables
pub async fn process_env(state: ChampStateArc) -> Result<()> {
    if let Ok(user) = env::var(CHAMP_DEBUG_CREATE_SUPERADMIN) {
        if let Some((username, password)) = user.split_once("::") {
            create_user::run(&state, username, password, vec!["superadmin".to_string()]).await?;
        }
    }

    if let Ok(peers) = env::var(CHAMP_INITIAL_PEERS) {
        let peers = peers.split(',').map(|s| s.to_string());
        let mut config = state.config.write().await;
        config.consensus.initial_peers.extend(peers)
    }

    if env::var(CHAMP_GENERATE_JWT_KEYS).is_ok() && {
        let admin = &state.config.read().await.admin;
        admin.jwt_private_key.is_none() || admin.jwt_public_key.is_none()
    } {
        generate_jwt_keys(&state).await?;
    }

    if let Ok(primary_wallet_password) = env::var(CHAMP_PRIMARY_WALLET_PASSWORD) {
        let mut config = state.config.write().await;
        let mut wallet_manager = state.wallet_manager.write().await;

        if env::var(CHAMP_GENERATE_PRIMARY_WALLET).is_ok() {
            config.consensus.primary_wallet = Some(wallet_manager.create_wallet(&primary_wallet_password).await?);
            config.write()?;
        }

        match config.consensus.primary_wallet.clone() {
            Some(primary_wallet) => wallet_manager.unlock_wallet(primary_wallet, &primary_wallet_password).await?,
            None => return Err(anyhow!("{CHAMP_PRIMARY_WALLET_PASSWORD} defined but no primary wallet to unlock. Specify primary wallet in config.consensus.primary_wallet"))
        }
    }

    Ok(())
}
