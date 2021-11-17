use crate::storage::Database;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::{blockpool::BlockpoolClient, config::Config};

#[derive(Debug)]
pub struct ChampState {
    pub db: Mutex<Box<dyn Database>>,
    pub config: RwLock<Config>,
    pub blockpool_client: BlockpoolClient,
}

impl ChampState {
    pub fn new(db: Box<dyn Database>, config: Config, blockpool_client: BlockpoolClient) -> ChampStateArc {
        Arc::new(Self {
            db: Mutex::new(db),
            config: RwLock::new(config),
            blockpool_client,
        })
    }

    #[cfg(test)]
    pub async fn mock() -> ChampStateArc {
        use crate::{blockpool::Blockpool, storage};

        let mut pool = Blockpool::new();
        let blockpool_client = pool.get_client();
        tokio::spawn(async move { pool.start().await });

        let db = Mutex::new(
            storage::new(&storage::DatabaseConfig {
                kind: storage::Databases::Mock,
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        Arc::new(Self {
            db,
            config: RwLock::new(Config::default()),
            blockpool_client,
        })
    }
}

pub type ChampStateArc = Arc<ChampState>;
