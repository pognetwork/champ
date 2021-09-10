use std::sync::Arc;
use storage::Database;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ChampState {
    pub db: Box<dyn Database>,
}

pub type ChampStateMutex = Arc<Mutex<ChampState>>;
