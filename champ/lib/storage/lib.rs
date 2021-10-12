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

/// Represents a generic storage backend
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
    #[error("data not found")]
    DataNotFound,
    #[error("fetch ID failed")]
    GetIDFailed,
    #[error("invalid database kind")]
    InvalidKind,
    #[error("no last block")]
    NoLastBlock,
    #[error("db insert failed at {0}")]
    DBInsertFailed(u32),
    #[error("this Error: {0}")]
    Specific(String),
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

    async fn get_block_by_id(&self, block_id: api::BlockID) -> Result<&api::Block, DatabaseError>;
    async fn get_block_by_height(
        &self,
        account_id: api::AccountID,
        block_height: &u64,
    ) -> Result<Option<&api::Block>, DatabaseError>;
    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<&api::Transaction, DatabaseError>;

    /// Finds the latest block for a given address
    ///
    /// Only includes confirmed blocks
    async fn get_latest_block_by_account(&self, acc_id: api::AccountID) -> Result<&api::Block, DatabaseError>;

    /// Finds the latest block for a given address before a given date
    ///
    /// Set limit to 0 to keep looking until an accounts first transaction
    async fn get_latest_block_by_account_before(
        &self,
        account_id: api::AccountID,
        // go back starting at this timestamp
        unix_from: u64,

        // don't go further than this timestamp
        unix_limit: u64,
    ) -> Result<Option<&api::Block>, DatabaseError>;

    // get_account_delegate finds out if an account is delegating their power to someone else
    async fn get_account_delegate(&self, account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError>;

    /// Finds who is delegating power to an account
    async fn get_delegates_by_account(&self, account_id: api::AccountID) -> Result<Vec<api::AccountID>, DatabaseError>;

    /// Adds a new block to the database
    async fn add_block(&mut self, block: api::Block) -> Result<(), DatabaseError>;
}
