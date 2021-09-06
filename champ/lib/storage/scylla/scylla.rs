use crate::{Database, DatabaseConfig};
use anyhow::Result;
use async_trait::async_trait;
use scylla::{Session, SessionBuilder};

pub struct Scylla {
    session: Option<Session>,
}

impl Scylla {
    pub fn new() -> Self {
        Self { session: None }
    }
}

#[async_trait]
impl Database for Scylla {
    async fn init(&mut self, cfg: &DatabaseConfig) -> Result<()> {
        self.session = Some(SessionBuilder::new().known_node(&cfg.uri).build().await?);
        Ok(())
    }
}
