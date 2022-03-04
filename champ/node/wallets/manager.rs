use crate::config::{read_file, write_file};
use crate::{config::WalletManagerConfig, state::ChampStateArc};
use anyhow::Result;
use lulw;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: UserName,
    pub locked: bool,
    pub account_address: String,
    pub authorized_roles: Vec<String>, // users with this role can access this account using our api
    pub private_key: Option<Vec<u8>>,  // Option for ease of use
}

type AccountAddress = String;
type UserName = String;

impl Wallet {
    pub fn lock(&mut self) {
        self.private_key = None;
        self.locked = true;
    }
}

#[derive(Debug)]
pub struct WalletManager {
    state: Option<ChampStateArc>,
    wallets: HashMap<AccountAddress, Wallet>,
    #[allow(dead_code)]
    config: WalletManagerConfig,
    index: HashMap<AccountAddress, UserName>,
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new(WalletManagerConfig::default())
    }
}

#[derive(Error, Debug)]
pub enum WalletManagerError {
    #[error("IO-Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Wallet error: {0}")]
    WalletError(#[from] lulw::WalletError),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("error with reading wallet: {0}")]
    ReadError(String),
    #[error("Wallet not found")]
    NotFoundError,
    #[error("error with unlocking wallet: {0}")]
    UnlockError(String),
    #[error("error with creating wallet")]
    CreateWalletError,
    #[error("error with creating wallet")]
    UserAlreadyHasWalletError,
    #[error(transparent)]
    SerializationError(#[from] serde_json::Error),
}

// Handles wallet interaction, locking, unlocking and creating.
// Saves wallets at data/wallets/[account_address].json
// Stores index to match wallets to users at data/wallets/index.json
impl WalletManager {
    pub fn new(config: WalletManagerConfig) -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config,
            index: HashMap::new(),
        }
    }

    pub fn mock() -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config: WalletManagerConfig::default(),
            index: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), WalletManagerError> {
        let mut path = self.get_base_path().await?;

        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(WalletManagerError::IOError)?;
        }

        path.push("index.json");
        if path.exists() {
            self.index = self.read_index().await?;
        } else {
            write_file(path, "").map_err(WalletManagerError::IOError)?;
            self.index = HashMap::new();
        }

        let path = self.get_wallets_path().await?;
        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(WalletManagerError::IOError)?;
        }

        self.initialize_wallets().await?;
        Ok(())
    }

    // create a wallet from passphrase and user_name, write the wallet to disk and add it to the index and wallet hashmap
    pub async fn create_wallet(
        &mut self,
        password: String,
        user_name: UserName,
    ) -> Result<AccountAddress, WalletManagerError> {
        let (wallet, account_address) = lulw::generate_wallet(password).map_err(WalletManagerError::WalletError)?;

        let mut path = self.get_wallets_path().await?;
        path.push(format!("{}.json", &account_address));

        write_file(path, &wallet).map_err(WalletManagerError::IOError)?;
        self.create_index_entry(account_address.clone(), user_name.clone()).await?;
        let wallet = Wallet {
            name: user_name,
            locked: true,
            account_address: account_address.clone(),
            authorized_roles: Vec::new(),
            private_key: None,
        };

        self.wallets.insert(account_address.clone(), wallet);
        Ok(account_address)
    }

    // deletes wallet from the FS, and both indeces
    pub async fn delete_wallet(&mut self, account_address: AccountAddress) -> Result<()> {
        self.delete_index_entry(&account_address).await?;
        self.wallets.remove(&account_address);

        let mut path = self.get_wallets_path().await?;
        path.push(account_address + ".json");
        fs::remove_file(path)?;
        Ok(())
    }

    // updates the user name associated with the wallet
    pub async fn rename_wallet(
        &mut self,
        account_address: AccountAddress,
        user_name: UserName,
    ) -> Result<(), WalletManagerError> {
        self.index.insert(account_address, user_name);
        self.write_index().await
    }

    pub async fn unlock_wallet(
        &mut self,
        account_address: AccountAddress,
        password: String,
    ) -> Result<(), WalletManagerError> {
        let wallet_file = self.read_wallet_file(&account_address).await?;
        let wallet = self.wallets.get_mut(&account_address).ok_or(WalletManagerError::NotFoundError)?;
        if !(wallet.locked) {
            return Ok(());
        }

        wallet.private_key =
            Some(lulw::unlock_wallet(&wallet_file, password).map_err(WalletManagerError::WalletError)?);

        wallet.locked = false;
        Ok(())
    }

    // reads all wallets from the FS
    async fn initialize_wallets(&mut self) -> Result<(), WalletManagerError> {
        let path = self.get_wallets_path().await?;
        let paths = std::fs::read_dir(path).map_err(WalletManagerError::IOError)?;
        for path in paths {
            let path = path.map_err(WalletManagerError::IOError)?.path();

            let file_name = path
                .file_name()
                .ok_or_else(|| WalletManagerError::Unknown("invalid wallet path name".to_string()))?
                .to_string_lossy();

            let account_address: String = file_name.split('.').collect::<Vec<&str>>()[0].to_owned();

            let user_name = self
                .index
                .get(&account_address)
                .ok_or_else(|| WalletManagerError::Unknown("no user owns this wallet".to_string()))?;

            let wallet = Wallet {
                name: user_name.clone(),
                locked: true,
                account_address: account_address.clone(),
                authorized_roles: Vec::new(),
                private_key: None,
            };

            self.wallets.insert(account_address.to_string(), wallet);
        }

        Ok(())
    }

    // adds a wallet to the index
    async fn create_index_entry(
        &mut self,
        account_address: AccountAddress,
        user_name: UserName,
    ) -> Result<(), WalletManagerError> {
        self.index.insert(account_address, user_name);
        self.write_index().await
    }

    // removes the index of the wallet
    async fn delete_index_entry(&mut self, account_address: &AccountAddress) -> Result<(), WalletManagerError> {
        self.index.remove(account_address);
        self.write_index().await
    }

    // reads the wallet from the FS
    #[allow(dead_code)]
    async fn read_wallet_file(&self, account_address: &AccountAddress) -> Result<String, WalletManagerError> {
        let mut path = self.get_wallets_path().await?;
        path.push(account_address.to_owned() + ".json");
        Ok(read_file(path)?)
    }

    // writes the index as json to file
    // inefficient to write the entire file when a new entry is added, maybe small DB? or optimize write
    async fn write_index(&self) -> Result<(), WalletManagerError> {
        let json = serde_json::to_string_pretty(&self.index)?;
        let mut path = self.get_base_path().await?;
        path.push("index.json");
        Ok(write_file(path, &json)?)
    }

    // reads the index.json and deserializes into self.index
    async fn read_index(&self) -> Result<HashMap<AccountAddress, UserName>, WalletManagerError> {
        let mut path = self.get_base_path().await?;
        path.push("index.json");
        let content = read_file(path.clone())?;

        let res = match content.as_str() {
            "" => HashMap::new(),
            _ => serde_json::from_str(&content)?,
        };

        Ok(res)
    }

    // returns pog_path\data\walletmanager
    async fn get_base_path(&self) -> Result<PathBuf, WalletManagerError> {
        let champ_state =
            self.state.clone().ok_or_else(|| WalletManagerError::Unknown("state not loaded".to_string()))?;

        let config = champ_state.config.read().await;

        let base_path =
            config.data_path.clone().ok_or_else(|| WalletManagerError::Unknown("unknown base path".to_string()))?;

        Ok(PathBuf::from(format!("{base_path}\\walletmanager")))
    }

    #[inline]
    // returns base_path\wallets
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

    // creates a wallet manager where the config path is change to /test/
    async fn get_wallet_manager() -> Result<WalletManager, WalletManagerError> {
        // this is big yikes, but the mocked champstated doesn't have a default datapath
        let mut wallet_manager = WalletManager::new(WalletManagerConfig::default());
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
        let username = "Malox".to_string();
        let password = "1234".to_string();
        let mut wallet_manager = get_wallet_manager().await.unwrap();

        // act
        let _ = wallet_manager.initialize().await;
        let account_address =
            wallet_manager.create_wallet(password.clone(), username.clone()).await.expect("Creating wallet failed");

        // assert
        assert!(wallet_manager.wallets.contains_key(&account_address));

        let wallet = wallet_manager.wallets.get(&account_address).expect("no wallet found").clone();
        assert_eq!(wallet.account_address, account_address);
        assert_eq!(wallet.name, username);

        assert!(wallet_manager.index.contains_key(&account_address));
        assert_eq!(wallet_manager.index[&account_address], username);

        // unlock wallet and check if the password unlocked it
        let result = wallet_manager.unlock_wallet(wallet.account_address, password).await;
        result.expect("Couldn't unlock wallet");
        let wallet = wallet_manager.wallets.get(&account_address).expect("no wallet found").clone();
        let private_key = wallet.private_key;
        assert!(private_key.is_some())
    }

    #[tokio::test]
    async fn delete_wallet() {
        let password = "1234".to_string();
        let username = "Malox".to_string();
        let mut wallet_manager = get_wallet_manager().await.unwrap();
        let account_address = wallet_manager.create_wallet(password, username).await.expect("Couldn't create wallet");

        // check if valid was indeed created
        assert!(wallet_manager.wallets.contains_key(account_address.as_str()));
        assert!(wallet_manager.index.contains_key(account_address.as_str()));
        let wallet = wallet_manager.read_wallet_file(&account_address).await.expect("Couldn't read wallet");
        assert!(!wallet.is_empty());

        // delete and check if it is gone
        wallet_manager.delete_wallet(account_address.clone()).await.expect("Couldn't delete wallet");
        assert!(!wallet_manager.wallets.contains_key(account_address.as_str()));
        assert!(!wallet_manager.index.contains_key(account_address.as_str()));
        let wallet = wallet_manager
            .read_wallet_file(&account_address)
            .await
            .expect_err("Could read wallet although it should be deleted");

        matches!(wallet, WalletManagerError::ReadError(_));
    }
}
