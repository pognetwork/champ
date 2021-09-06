use std::sync::Arc;
use tokio::sync::Mutex;
use storage::Database;

#[derive(Debug)]
pub struct ChampState {
    pub db: Box<dyn Database>,
}

pub type ChampStateMutex = Arc<Mutex<ChampState>>;
