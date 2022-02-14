use crate::config::{read_file, write_file};
use crate::{config::WalletManagerConfig, state::ChampStateArc};
use lulw;
use pog_proto::api::AccountID;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Wallet {
    pub name: String,
    pub locked: bool,
    pub account_id: AccountID,
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
    wallets: HashMap<AccountID, Wallet>,
    #[allow(dead_code)]
    config: WalletManagerConfig,
    index: Vec<WalletIndex>, //make Hashmap
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new(WalletManagerConfig::default())
    }
}

//Handles wallets interaction, locking, unlocking and creating.
//Saves wallets at data/wallets/[account_address].json
//Stores index to match wallets to users at data/wallets/index.json
impl WalletManager {
    pub fn new(config: WalletManagerConfig) -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config,
            index: Vec::new(),
        }
    }

    pub fn mock() -> Self {
        Self {
            state: None,
            wallets: HashMap::new(),
            config: WalletManagerConfig::default(),
            index: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) {
        let mut path = self.get_path().await;
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Coulnd't Write 'wallets' directory to drive");
        }

        path.push("index.json");
        if !path.exists() {
            write_file(path, "").expect("Coulnd't write empty index.json to drive");
            self.index = Vec::new();
        } else {
            self.index = self.read_index().await;
        }

        //self.generate_wallet("1234".to_string(), "Malox".to_string()).await;
    }

    pub async fn generate_wallet(&mut self, password: String, user_name: String) -> Result<(), lulw::WalletError> {
        let (wallet, account_address) = lulw::generate_wallet(password)?;
        let mut path = self.get_path().await;

        path.push(format!("{}.json", account_address));
        write_file(path, &wallet).expect("Couldn't write wallet to file");
        self.add_to_index(account_address, user_name).await;
        Ok(())
    }

    //reads the index.json and deserializes into self.index
    async fn read_index(&self) -> Vec<WalletIndex> {
        let mut path = self.get_path().await;
        path.push("index.json");
        let content = read_file(path).expect("Couldn't read wallet index");
        match content.as_str() {
            "" => Vec::new(),
            _ => serde_json::from_str(&content).expect("Couldn't parse index.json"),
        }
    }

    //writes the index as json to file
    //inefficient to write the entire file when a new entry is added, maybe small DB? or optimize write
    async fn write_index(&self) {
        let json = serde_json::to_string_pretty(&self.index).expect("Failed to serialize wallet index to json");
        let mut path = self.get_path().await;
        path.push("\\index.json");
        write_file(path, &json).expect("Couldn't write index to file");
    }

    //adds a wallet to the index
    async fn add_to_index(&mut self, account_address: String, user_name: String) {
        //let account_address = wallet.account_id.encode_zbase().expect("Can't encode wallet account address to zbase");
        self.index.push(WalletIndex {
            account_address,
            user_name,
        });
        self.write_index().await;
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
    fn get_wallets(&self) -> &HashMap<AccountID, Wallet>;
}

impl WalletInterface for WalletManager {
    fn get_wallets(&self) -> &HashMap<AccountID, Wallet> {
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
