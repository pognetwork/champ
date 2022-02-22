use crate::config::{read_file, write_file};
use crate::{config::WalletManagerConfig, state::ChampStateArc};
use anyhow::Result;
use lulw;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: UserName,
    pub locked: bool,
    pub account_address: String,
    pub authorized_roles: Vec<String>, // users with this role can access this account using our api
    private_key: Option<Vec<u8>>,      //Option for ease of use
}

type AccountAddress = String;
type UserName = String;

impl Wallet {
    pub fn unlock(&mut self, wallet: &str, password: String) -> Result<(), lulw::WalletError> {
        if !&self.locked {
            return Ok(());
        }

        self.private_key = Some(lulw::unlock_wallet(wallet, password)?);
        self.locked = false;
        Ok(())
    }

    pub fn lock(&mut self) {
        self.private_key = None;
        self.locked = true;
    }
}

//#[derive(Debug)]
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

//Handles wallet interaction, locking, unlocking and creating.
//Saves wallets at data/wallets/[account_address].json
//Stores index to match wallets to users at data/wallets/index.json
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

    pub async fn initialize(&mut self) -> Result<(), lulw::WalletError> {
        let mut path = self.get_base_path().await;

        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Coulnd't Write 'walletmanager' directory to drive");
        }

        path.push("index.json");
        if !path.exists() {
            write_file(path, "").expect("Coulnd't write empty index.json to drive");
            self.index = HashMap::new();
        } else {
            self.index = self.read_index().await;
        }

        let path = self.get_wallets_path().await;
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Coulnd't Write 'wallets' directory to drive");
        }

        self.initialize_wallets().await;

        self.create_wallet("1234".to_string(), "Malox".to_string()).await.expect("I hope this doesn't fail");
        self.create_wallet("123456".to_string(), "Alex".to_string()).await.expect("I hope this doesn't fail");

        Ok(())
    }

    //create a wallet from passphrase and user_name, write the wallet to disk and add it to the index and wallet hashmap
    pub async fn create_wallet(
        &mut self,
        password: String,
        user_name: UserName,
    ) -> Result<AccountAddress, lulw::WalletError> {
        let (wallet, account_address) = lulw::generate_wallet(password)?;

        let mut path = self.get_wallets_path().await;
        path.push(format!("{}.json", &account_address));

        //keep address as string?
        //let account_address = decode(account_address).expect("Couldn't parse valid account address to u8 vec");
        //let account_address = AccountID::try_from(account_address).unwrap();
        println!("{}", path.to_str().unwrap());
        write_file(path, &wallet).expect("Couldn't write wallet to file");
        self.create_index_entry(account_address.clone(), user_name.clone()).await;
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

    //deletes wallet from the FS, and both indeces
    pub async fn delete_wallet(&mut self, account_address: AccountAddress) -> Result<()> {
        self.delete_index_entry(&account_address).await;
        self.wallets.remove(&account_address);

        let mut path = self.get_wallets_path().await;
        path.push(account_address + ".json");
        fs::remove_file(path)?;
        Ok(())
    }

    //updates the user name associated with the wallet
    pub async fn rename_wallet(&mut self, account_address: AccountAddress, user_name: UserName) {
        self.index.insert(account_address, user_name);
        self.write_index().await;
    }

    //reads all wallets from the FS
    async fn initialize_wallets(&mut self) {
        let path = self.get_wallets_path().await;
        let paths = std::fs::read_dir(path).expect("Couldn't read Directory");
        for path in paths {
            let path = path.expect("").path();

            let file_name = path.file_name().expect("file doesn't exist?").to_string_lossy();
            let account_address: String = file_name.split('.').collect::<Vec<&str>>()[0].to_owned();

            let user_name = self.index.get(&account_address).expect("no user owns this wallet");
            let wallet = Wallet {
                name: user_name.clone(),
                locked: true,
                account_address: account_address.clone(),
                authorized_roles: Vec::new(),
                private_key: None,
            };

            self.wallets.insert(account_address.to_string(), wallet);
        }
    }

    //adds a wallet to the index
    async fn create_index_entry(&mut self, account_address: AccountAddress, user_name: UserName) {
        self.index.insert(account_address, user_name);
        self.write_index().await;
    }

    //removes the index of the wallet
    async fn delete_index_entry(&mut self, account_address: &AccountAddress) {
        self.index.remove(account_address);
        self.write_index().await;
    }

    //reads the wallet from the FS
    #[allow(dead_code)]
    async fn read_wallet_file(&self, account_address: &AccountAddress) -> String {
        let mut path = self.get_wallets_path().await;
        path.push(account_address.to_owned() + ".json");
        read_file(path).expect("Couldn't read Wallet File")
    }

    //writes the index as json to file
    //inefficient to write the entire file when a new entry is added, maybe small DB? or optimize write
    async fn write_index(&self) {
        let json = serde_json::to_string_pretty(&self.index).expect("Failed to serialize wallet index to json");
        let mut path = self.get_base_path().await;
        path.push("index.json");
        write_file(path, &json).expect("Couldn't write index to file");
    }

    //reads the index.json and deserializes into self.index
    async fn read_index(&self) -> HashMap<AccountAddress, UserName> {
        let mut path = self.get_base_path().await;
        path.push("index.json");
        let content = read_file(path).expect("Couldn't read wallet index");
        match content.as_str() {
            "" => HashMap::new(),
            _ => serde_json::from_str(&content).expect("Couldn't parse index.json"),
        }
    }

    //returns pog_path\data\walletmanager
    async fn get_base_path(&self) -> PathBuf {
        let champ_state = self.state.clone().expect("ChampStateArc should be Some but wasn't");
        let config = champ_state.config.read().await;
        let base_path = config.data_path.clone().expect("no data path provided");
        PathBuf::from(format!("{}\\walletmanager", base_path))
    }

    #[inline]
    //returns base_path\wallets
    async fn get_wallets_path(&self) -> PathBuf {
        let mut path = self.get_base_path().await;
        path.push("wallets");
        path
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

    #[tokio::test]
    async fn create_wallet() {
        //prepare
        let username = "Malox".to_string();
        let password = "1234".to_string();

        //this is big yikes, but the mocked champstated doesn't have a default datapath
        let mut wallet_manager = WalletManager::new(WalletManagerConfig::default());
        let champstate = ChampState::mock().await;

        let mut path = Config::default().get_path().unwrap();
        path.pop();
        path.push("test");

        let mut write_guard = champstate.config.write().await;
        write_guard.data_path = Some(path.to_str().expect("couldn't convert to string").to_string());
        std::mem::drop(write_guard);
        wallet_manager.add_state(champstate);

        //act
        let _ = wallet_manager.initialize().await;
        let account_address =
            wallet_manager.create_wallet(password.clone(), username.clone()).await.expect("Creating wallet failed");

        //assert
        assert!(wallet_manager.wallets.contains_key(&account_address));

        let wallet = &wallet_manager.wallets[&account_address];
        assert_eq!(wallet.account_address, account_address);
        assert_eq!(wallet.name, username);

        assert!(wallet_manager.index.contains_key(&account_address));
        assert_eq!(wallet_manager.index[&account_address], username);

        //Nonce differs somehow check that the wallet is the right one
        //let mut path = wallet_manager.get_wallets_path().await;
        //path.push(account_address + ".json");
        //let file = read_file(path).expect("Couldn't read wallet from disc");
        //let (wallet, _) = lulw::generate_wallet(password).expect("couldn't create wallet");
        //assert_eq!(file, wallet);
    }
}
