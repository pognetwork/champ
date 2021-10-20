use std::sync::Arc;
use storage::Database;
use tokio::sync::{Mutex, RwLock};

use crate::config::Config;

#[derive(Debug)]
pub struct ChampState {
    pub db: Mutex<Box<dyn Database>>,
    pub config: RwLock<Config>,
}

impl ChampState {
    pub fn new(db: Box<dyn Database>, config: Config) -> Arc<Self> {
        Arc::new(Self {
            db: Mutex::new(db),
            config: RwLock::new(config),
        })
    }

    #[cfg(test)]
    pub async fn mock() -> Arc<Self> {
        Arc::new(Self {
            db: Mutex::new(
                storage::new(&storage::DatabaseConfig {
                    kind: storage::Databases::Mock,
                    uri: None,
                    path: None,
                })
                .await
                .unwrap(),
            ),
            config: RwLock::new(Config::default()),
        })
    }
}

pub type ChampStateMutex = Arc<ChampState>;
