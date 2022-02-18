use crate::config::{read_file, write_file};
use crate::{config::WalletManagerConfig, state::ChampStateArc};
use anyhow::Result;
use lulw;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: String,
    pub locked: bool,
    pub account_address: String,
    pub authorized_roles: Vec<String>, // users with this role can access this account using our api
    private_key: Option<Vec<u8>>,      //Option for ease of use
}

impl Wallet {
    #[inline]
    pub fn unlock(&mut self, wallet: &str, password: String) -> Result<(), lulw::WalletError> {
        if !&self.locked {
            return Ok(());
        }

        self.private_key = Some(lulw::unlock_wallet(wallet, password)?);
        self.locked = false;
        Ok(())
    }

    #[inline]
    pub fn lock(&mut self) {
        self.private_key = None;
        self.locked = true;
    }
}

//#[derive(Debug)]
pub struct WalletManager {
    state: Option<ChampStateArc>,
    wallets: HashMap<String, Wallet>,
    #[allow(dead_code)]
    config: WalletManagerConfig,
    index: HashMap<String, String>, //change String to AccountID
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
        let mut path = self.get_path().await;
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Coulnd't Write 'wallets' directory to drive");
        }

        path.push("index.json");
        if !path.exists() {
            write_file(path, "").expect("Coulnd't write empty index.json to drive");
            self.index = HashMap::new();
        } else {
            self.index = self.read_index().await;
        }

        self.create_wallet("1234".to_string(), "Malox".to_string()).await.expect("I hope this doesn't fail");
        Ok(())
    }

    //create a wallet from passphrase and user_name, write the wallet to desk and add it to the index and wallet hashmap
    pub async fn create_wallet(&mut self, password: String, user_name: String) -> Result<(), lulw::WalletError> {
        let (wallet, account_address) = lulw::generate_wallet(password)?;

        let mut path = self.get_path().await;
        path.push(format!("{}.json", &account_address));

        //keep address as string?
        //let account_address = decode(account_address).expect("Couldn't parse valid account address to u8 vec");
        //let account_address = AccountID::try_from(account_address).unwrap();

        write_file(path, &wallet).expect("Couldn't write wallet to file");
        self.create_index_entry(account_address.clone(), user_name.clone()).await;
        let wallet = Wallet {
            name: user_name,
            locked: true,
            account_address: account_address.clone(),
            authorized_roles: Vec::new(),
            private_key: None,
        };

        self.wallets.insert(account_address, wallet);
        Ok(())
    }

    pub async fn delete_wallet(&mut self, account_address: String) -> Result<()> {
        self.wallets.remove(&account_address);
        self.index.remove(&account_address);

        let mut path = self.get_path().await;
        path.push(account_address + ".json");
        fs::remove_file(path)?;
        Ok(())
    }

    //adds a wallet to the index
    async fn create_index_entry(&mut self, account_address: String, user_name: String) {
        self.index.insert(account_address, user_name);
        self.write_index().await;
    }

    #[allow(unused_variables)]
    #[allow(dead_code)]
    async fn update_index_entry(&mut self, account_address: String, user_name: String) {}

    #[allow(unused_variables)]
    #[allow(dead_code)]
    async fn delete_index_entry(&mut self, account_address: String) {}

    //writes the index as json to file
    //inefficient to write the entire file when a new entry is added, maybe small DB? or optimize write
    async fn write_index(&self) {
        let json = serde_json::to_string_pretty(&self.index).expect("Failed to serialize wallet index to json");
        let mut path = self.get_path().await;
        path.push("index.json");
        write_file(path, &json).expect("Couldn't write index to file");
    }

    //reads the index.json and deserializes into self.index
    async fn read_index(&self) -> HashMap<String, String> {
        let mut path = self.get_path().await;
        path.push("index.json");
        let content = read_file(path).expect("Couldn't read wallet index");
        match content.as_str() {
            "" => HashMap::new(),
            _ => serde_json::from_str(&content).expect("Couldn't parse index.json"),
        }
    }

    //returns base_path\data\wallets
    async fn get_path(&self) -> PathBuf {
        let champ_state = self.state.clone().expect("ChampStateArc should be Some but wasn't");
        let config = champ_state.config.read().await;
        let base_path = config.data_path.clone().expect("no data path provided");
        PathBuf::from(format!("{}\\wallets", base_path))
    }

    pub fn add_state(&mut self, state: ChampStateArc) {
        self.state = Some(state);
    }
}

pub trait WalletInterface {
    fn get_wallets(&self) -> &HashMap<String, Wallet>;
}

impl WalletInterface for WalletManager {
    fn get_wallets(&self) -> &HashMap<String, Wallet> {
        &self.wallets
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct WalletIndex {
    #[serde(rename = "a")]
    pub account_address: String,
    #[serde(rename = "u")]
    pub user_name: String,
}
