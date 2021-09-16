use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api;

use crate::{Database, DatabaseConfig};

#[derive(Default, Debug)]
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

    async fn get_block_by_id(&self, _block_id: &str) -> Result<&api::Block, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }
    async fn get_transaction_by_id(&self, _transaction_id: &str) -> Result<&api::Transaction, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_latest_block_by_account(&self, _account_id: &str) -> Result<&api::Block, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn add_block(&mut self, _block: api::Block) -> Result<(), DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_block_by_height(&self, _account_id: &str, _block_height: &u64) -> Result<&api::Block, DatabaseError> {
        unimplemented!()
    }
    async fn get_account_delegate(&self, _account_id: &str) -> Result<Option<&str>, DatabaseError> {
        unimplemented!()
    }

    async fn get_delegates_by_account(&self, _account_id: &str) -> Result<Vec<&str>, DatabaseError> {
        unimplemented!()
    }

    async fn get_latest_block_by_account_before(
        &self,
        _account_id: &str,
        _unix_from: u64,
        _unix_limit: u64,
    ) -> Result<Option<&api::Block>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }
}
