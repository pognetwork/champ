use crate::Database;
use anyhow::Result;
use scylla::{Session, SessionBuilder};

pub struct Scylla {
    session: Session,
}

impl Scylla {
    async fn connect(mut self, uri: &str) -> Result<()> {
        self.session = SessionBuilder::new().known_node(uri).build().await?;

        Ok(())
    }
}

impl Database for Scylla {}
