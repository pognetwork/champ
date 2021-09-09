use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api;
use thiserror::Error;

pub mod mock;
#[cfg(feature = "backend-rocksdb")]
pub mod rocksdb;
#[cfg(feature = "backend-scylla")]
pub mod scylla;

#[non_exhaustive]
pub enum Databases {
    Mock,
    #[cfg(feature = "backend-rocksdb")]
    RocksDB,
    #[cfg(feature = "backend-scylla")]
    Scylla,
}

pub struct DatabaseConfig<'a> {
    pub kind: Databases,
    pub uri: &'a str,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("unknown database error")]
    Unknown,
    #[error("invalid database kind")]
    InvalidKind,
}

pub async fn new(cfg: &DatabaseConfig<'_>) -> Result<Box<dyn Database>, DatabaseError> {
    let mut db: Box<dyn Database>;
    match cfg.kind {
        #[cfg(feature = "backend-rocksdb")]
        Databases::RocksDB => {
            db = Box::new(rocksdb::RocksDB::new());
            db.init(cfg).await.map_err(|_e| DatabaseError::Unknown)?;
        }
        #[cfg(feature = "backend-scylla")]
        Databases::Scylla => {
            db = Box::new(scylla::Scylla::new());
            db.init(cfg).await.map_err(|_e| DatabaseError::Unknown)?;
        }
        Databases::Mock => {
            db = Box::new(mock::MockDB::new());
            db.init(cfg).await.map_err(|_e| DatabaseError::Unknown)?;
        } // _ => return Err(DatabaseError::InvalidKind),
    }
    Ok(db)
}

impl Debug for dyn Database {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Database{{}}")
    }
}

#[async_trait]
// Send and sync are added because of async traits: https://github.com/dtolnay/async-trait#dyn-traits
pub trait Database: Send + Sync {
    async fn init(&mut self, config: &DatabaseConfig) -> Result<()>;

    async fn get_block_by_id(&self, _block_id: &str) -> Result<&api::Block, DatabaseError>;

    async fn get_transaction_by_id(
        &self,
        _transaction_id: &str,
    ) -> Result<&api::Transaction, DatabaseError>;

    async fn get_latest_block_by_account(
        &self,
        _acc_id: &str,
    ) -> Result<&api::Block, DatabaseError>;

    async fn get_account_by_id(
        &self,
        _account_id: &str,
    ) -> Result<&api::PublicAccount, DatabaseError>;

    async fn add_block(&mut self, _block: api::Block) -> Result<(), DatabaseError>;
}
