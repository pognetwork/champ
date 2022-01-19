use crate::{config::WalletManagerConfig, state::ChampStateArc};
use pog_proto::api::AccountID;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: String,
    pub locked: bool,
    pub account_id: String,
    pub authorized_roles: Vec<String>, // users with this role can access this account using our api
}

#[derive(Debug)]
pub struct WalletManager {
    state: Option<ChampStateArc>,
    wallets: HashMap<AccountID, Wallet>,
    config: WalletManagerConfig,
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new(WalletManagerConfig::default())
    }
}

impl WalletManager {
    pub fn new(config: WalletManagerConfig) -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config,
        }
    }

    pub fn mock() -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config: WalletManagerConfig::default(),
        }
    }

    pub fn add_state(&mut self, state: ChampStateArc) {
        self.state = Some(state);
    }
}

pub trait WalletInterface {
    fn get_wallets(&self) -> &HashMap<AccountID, Wallet>;
}

impl WalletInterface for WalletManager {
    fn get_wallets(&self) -> &HashMap<AccountID, Wallet> {
        &self.wallets
    }
}
