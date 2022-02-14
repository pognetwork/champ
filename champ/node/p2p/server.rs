use crate::state::ChampStateArc;

pub struct P2PServer {
    #[allow(dead_code)]
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
