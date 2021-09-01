use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct ChampState {
    pub username: String,
}

pub type ChampStateMutex = Arc<Mutex<ChampState>>;
