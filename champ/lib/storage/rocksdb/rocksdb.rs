use anyhow::Result;
use async_trait::async_trait;

use crate::{Database, DatabaseConfig};

pub struct RocksDB {
    db: Option<rocksdb::DB>,
}

impl RocksDB {
    pub fn new() -> Self {
        Self { db: None }
    }
}

#[async_trait]
impl Database for RocksDB {
    async fn init(&mut self, _: &DatabaseConfig) -> Result<()> {
        unimplemented!("a")
    }
}
