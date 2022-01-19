use crate::state::ChampStateArc;

pub struct P2PServer {
    state: ChampStateArc,
}

impl P2PServer {
    pub fn new(state: ChampStateArc) -> Self {
        Self {
            state,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
