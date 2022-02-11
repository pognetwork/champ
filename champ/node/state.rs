use crate::{storage::Database, wallets::WalletManager};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::{blockpool::BlockpoolClient, config::Config};

#[derive(Debug)]
pub struct ChampState {
    pub db: Mutex<Box<dyn Database>>,
    pub config: RwLock<Config>,
    pub wallet_manager: RwLock<WalletManager>,
    pub blockpool_client: BlockpoolClient,
}

pub struct ChampStateArgs {
    pub db: Box<dyn Database>,
    pub config: RwLock<Config>,
    pub wallet_manager: RwLock<WalletManager>,
    pub blockpool_client: BlockpoolClient,
}

impl ChampState {
    pub fn new(args: ChampStateArgs) -> ChampStateArc {
        Arc::new(Self {
            db: Mutex::new(args.db),
            config: args.config,
            wallet_manager: args.wallet_manager,
            blockpool_client: args.blockpool_client,
        })
    }

    #[cfg(test)]
    pub async fn mock() -> ChampStateArc {
        use crate::{blockpool::Blockpool, storage};

        let mut pool = Blockpool::new();
        let blockpool_client = pool.get_client();

        let db = Mutex::new(
            storage::new(&storage::DatabaseConfig {
                kind: storage::Databases::Sled,
                temporary: Some(true),
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        let state = Arc::new(Self {
            db,
            config: RwLock::new(Config::default()),
            wallet_manager: RwLock::new(WalletManager::mock()),
            blockpool_client,
        });
        pool.add_state(state.clone());
        tokio::spawn(async move { pool.start().await });
        state
    }
}

pub type ChampStateArc = Arc<ChampState>;
