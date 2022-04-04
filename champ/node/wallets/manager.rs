use crate::config::{read_file, write_file};
use crate::state::ChampStateArc;
use anyhow::Result;
use encoding::account::parse_account_address_string;
use lulw;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zeroize::Zeroize;

type AccountAddress = String;
type WalletName = String;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub locked: bool,
    pub account_address: String,
    pub account_address_bytes: Vec<u8>,
    private_key: Option<Vec<u8>>,
}

impl Wallet {
    pub fn new(account_address: &str) -> Result<Self, WalletManagerError> {
        Ok(Wallet {
            account_address: account_address.to_string(),
            account_address_bytes: parse_account_address_string(&account_address)?,
            locked: true,
            private_key: None,
        })
    }

    pub fn lock(&mut self) {
        self.private_key.zeroize();
        self.private_key = None;
        self.locked = true;
    }

    pub fn public_key(&self) -> Result<[u8; 32], WalletManagerError> {
        match &self.private_key {
            Some(key) => crypto::signatures::ed25519::create_public_key(key)
                .map_err(|e| WalletManagerError::Unknown(format!("failed to generate public key: {e}"))),
            None => Err(WalletManagerError::Locked),
        }
    }

    pub fn set_private_key(&mut self, key: Vec<u8>) {
        self.private_key = Some(key);
        self.locked = false;
    }

    pub fn sign(&mut self, data: &[u8]) -> Result<[u8; 64], WalletManagerError> {
        match &self.private_key {
            Some(key) => crypto::signatures::ed25519::create_signature(data, key)
                .map_err(|e| WalletManagerError::Unknown(format!("failed to generate signature: {e}"))),
            None => Err(WalletManagerError::Locked),
        }
    }
}

#[derive(Debug, Default)]
pub struct WalletManager {
    state: Option<ChampStateArc>,
    wallets: HashMap<AccountAddress, Wallet>,
    names: HashMap<AccountAddress, WalletName>,
}

#[derive(Error, Debug)]
pub enum WalletManagerError {
    #[error("IO-Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Wallet error: {0}")]
    WalletError(#[from] lulw::WalletError),
    #[error("Account error: {0}")]
    AccountError(#[from] encoding::account::AccountError),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("error with reading wallet: {0}")]
    ReadError(String),
    #[error("Wallet not found")]
    NotFoundError,
    #[error("wallet is locked")]
    Locked,
    #[error("error with unlocking wallet: {0}")]
    UnlockError(String),
    #[error("error with creating wallet")]
    CreateWalletError,
    #[error("error with creating wallet")]
    UserAlreadyHasWalletError,
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

const INDEX_FILE: &str = "index.json";

/// Handles wallet interaction, locking, unlocking and creating.
/// Saves wallets at data/wallets/[account_address].json
/// Stores each wallets name at data/wallets/index.json
impl WalletManager {
    pub fn new() -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            names: HashMap::new(),
        }
    }

    pub fn mock() -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            names: HashMap::new(),
        }
    }

    pub async fn primary_wallet(&self) -> Option<&Wallet> {
        let state = self.state.clone()?;
        let config = &state.config.read().await;
        let primary_wallet = config.consensus.primary_wallet.as_ref()?;
        self.get_wallet(primary_wallet).await
    }

    pub async fn initialize(&mut self) -> Result<(), WalletManagerError> {
        let mut path = self.get_base_path().await?;

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        path.push(INDEX_FILE);
        if path.exists() {
            self.names = self.read_index().await?;
        } else {
            write_file(path, "")?;
            self.names = HashMap::new();
        }

        let path = self.get_wallets_path().await?;
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        self.initialize_wallets().await?;
        Ok(())
    }

    /// create a wallet from passphrase and user_name, write the wallet to disk and add it to the index and wallet hashmap
    pub async fn create_wallet(&mut self, password: &str) -> Result<AccountAddress, WalletManagerError> {
        let (wallet, account_address) = lulw::generate_wallet(password)?;

        let mut path = self.get_wallets_path().await?;
        path.push(format!("{}.json", &account_address));
        write_file(path, &wallet)?;

        let wallet = Wallet::new(&account_address)?;
        self.wallets.insert(account_address.clone(), wallet);
        Ok(account_address)
    }

    /// deletes wallet from the FS, and both indeces
    pub async fn delete_wallet(&mut self, account_address: &str) -> Result<()> {
        self.delete_index_entry(account_address).await?;
        self.wallets.remove(account_address);

        let mut path = self.get_wallets_path().await?;
        path.push(format!("{}.json", &account_address));
        fs::remove_file(path)?;
        Ok(())
    }

    /// updates the name associated with the wallet
    pub async fn get_wallet(&self, account_address: &str) -> Option<&Wallet> {
        self.wallets.get(account_address)
    }

    /// updates the name associated with the wallet
    pub async fn rename_wallet(&mut self, account_address: &str, wallet: &str) -> Result<(), WalletManagerError> {
        self.names.insert(account_address.to_string(), wallet.to_string());
        self.write_index().await
    }

    pub async fn unlock_wallet(
        &mut self,
        account_address: AccountAddress,
        password: &str,
    ) -> Result<(), WalletManagerError> {
        let wallet_file = self.read_wallet_file(&account_address).await?;
        let wallet = self.wallets.get_mut(&account_address).ok_or(WalletManagerError::NotFoundError)?;

        if !(wallet.locked) {
            return Ok(());
        }

        {
            let private_key = lulw::unlock_wallet(&wallet_file, password).map_err(WalletManagerError::WalletError)?;
            wallet.set_private_key(private_key);
        }

        Ok(())
    }

    /// reads all wallets from the FS
    async fn initialize_wallets(&mut self) -> Result<(), WalletManagerError> {
        let path = self.get_wallets_path().await?;
        let paths = std::fs::read_dir(path).map_err(WalletManagerError::IOError)?;
        for path in paths {
            let path = path.map_err(WalletManagerError::IOError)?.path();

            let file_name = path
                .file_name()
                .ok_or_else(|| WalletManagerError::Unknown("invalid wallet path".to_string()))?
                .to_string_lossy();

            let account_address: String = file_name.split('.').collect::<Vec<&str>>()[0].to_owned();
            let wallet = Wallet::new(&account_address)?;
            self.wallets.insert(account_address.to_string(), wallet);
        }

        Ok(())
    }

    /// removes the index of the wallet
    async fn delete_index_entry(&mut self, account_address: &str) -> Result<(), WalletManagerError> {
        self.names.remove(account_address);
        self.write_index().await
    }

    /// reads the wallet from the FS
    #[allow(dead_code)]
    async fn read_wallet_file(&self, account_address: &str) -> Result<String, WalletManagerError> {
        let mut path = self.get_wallets_path().await?;
        path.push(format!("{account_address}.json"));
        Ok(read_file(path)?)
    }

    /// writes the index as json to file
    /// inefficient to write the entire file when a new entry is added, maybe small DB? or optimize write
    async fn write_index(&self) -> Result<(), WalletManagerError> {
        let json = serde_json::to_string_pretty(&self.names)?;
        let mut path = self.get_base_path().await?;
        path.push(INDEX_FILE);
        Ok(write_file(path, &json)?)
    }

    /// reads the index.json and deserializes into self.index
    async fn read_index(&self) -> Result<HashMap<AccountAddress, WalletName>, WalletManagerError> {
        let mut path = self.get_base_path().await?;
        path.push(INDEX_FILE);
        let content = read_file(path.clone())?;

        let res = match content.as_str() {
            "" => HashMap::new(),
            _ => serde_json::from_str(&content)?,
        };

        Ok(res)
    }

    /// returns pog_path\data\walletmanager
    async fn get_base_path(&self) -> Result<PathBuf, WalletManagerError> {
        let champ_state =
            self.state.clone().ok_or_else(|| WalletManagerError::Unknown("state not loaded".to_string()))?;

        let config = champ_state.config.read().await;

        let base_path =
            config.data_path.clone().ok_or_else(|| WalletManagerError::Unknown("unknown base path".to_string()))?;

        Ok(Path::new(&base_path).join("walletmanager"))
    }

    #[inline]
    /// returns base_path\wallets
    async fn get_wallets_path(&self) -> Result<PathBuf, WalletManagerError> {
        let mut path = self.get_base_path().await?;
        path.push("wallets");
        Ok(path)
    }

    pub fn add_state(&mut self, state: ChampStateArc) {
        self.state = Some(state);
    }
}

pub trait WalletInterface {
    fn get_wallets(&self) -> &HashMap<AccountAddress, Wallet>;
}

impl WalletInterface for WalletManager {
    fn get_wallets(&self) -> &HashMap<AccountAddress, Wallet> {
        &self.wallets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ChampState;

    /// creates a wallet manager where the config path is change to /test/
    async fn get_wallet_manager() -> Result<WalletManager, WalletManagerError> {
        let mut wallet_manager = WalletManager::new();
        let champstate = ChampState::mock().await;

        // get the default path of the .toml pop and it add a test folder.
        let mut path = Config::default().get_path().unwrap();
        path.pop();
        path.push("test");

        let mut write_guard = champstate.config.write().await;
        write_guard.data_path = Some(
            path.to_str().ok_or_else(|| WalletManagerError::Unknown("invalid data path".to_string()))?.to_string(),
        );
        std::mem::drop(write_guard);
        wallet_manager.add_state(champstate);
        Ok(wallet_manager)
    }

    #[tokio::test]
    async fn create_wallet() {
        // prepare
        let wallet_name = "Malox";
        let password = "1234";
        let mut wallet_manager = get_wallet_manager().await.unwrap();

        // act
        let _ = wallet_manager.initialize().await;
        let account_address = wallet_manager.create_wallet(password).await.expect("Creating wallet failed");
        wallet_manager.rename_wallet(account_address.as_str(), wallet_name).await.expect("should rename wallet");

        // assert
        assert!(wallet_manager.wallets.contains_key(&account_address));

        let wallet = wallet_manager.wallets.get(&account_address).expect("no wallet found").clone();
        assert_eq!(wallet.account_address, account_address);

        assert!(wallet_manager.names.contains_key(&account_address));
        assert_eq!(wallet_manager.names[&account_address], wallet_name);

        // unlock wallet and check if the password unlocked it
        let result = wallet_manager.unlock_wallet(wallet.account_address, password).await;
        result.expect("Couldn't unlock wallet");
        let wallet = wallet_manager.wallets.get(&account_address).expect("no wallet found").clone();

        let private_key = wallet.private_key;
        assert!(private_key.is_some())
    }

    #[tokio::test]
    async fn delete_wallet() {
        let password = "1234";
        let wallet_name = "Malox";
        let mut wallet_manager = get_wallet_manager().await.unwrap();
        let account_address = wallet_manager.create_wallet(password).await.expect("Couldn't create wallet");
        wallet_manager.rename_wallet(account_address.as_str(), wallet_name).await.expect("should rename wallet");

        // check if valid was indeed created
        assert!(wallet_manager.wallets.contains_key(account_address.as_str()));
        assert!(wallet_manager.names.contains_key(account_address.as_str()));
        let wallet = wallet_manager.read_wallet_file(&account_address).await.expect("Couldn't read wallet");
        assert!(!wallet.is_empty());

        // delete and check if it is gone
        wallet_manager.delete_wallet(account_address.as_str()).await.expect("Couldn't delete wallet");
        assert!(!wallet_manager.wallets.contains_key(account_address.as_str()));
        assert!(!wallet_manager.names.contains_key(account_address.as_str()));
        let wallet = wallet_manager
            .read_wallet_file(&account_address)
            .await
            .expect_err("Could read wallet although it should be deleted");

        matches!(wallet, WalletManagerError::ReadError(_));
    }
}
