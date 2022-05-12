use std::fmt::Debug;

use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api::{self, AccountID};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "backend-sled")]
use super::sled;

#[allow(unused_imports)]
#[cfg(feature = "sql")]
use super::sql;

/// Represents a generic storage backend
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub enum Databases {
    #[cfg(feature = "backend-sqlite")]
    SQLite,
    #[cfg(feature = "backend-postgres")]
    Postgres,
    #[cfg(feature = "backend-mysql")]
    MySQL,
    #[cfg(feature = "backend-sled")]
    Sled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub kind: Databases,
    pub uri: Option<String>,

    // the database is deleted after it is dropped
    pub temporary: Option<bool>,

    /// absolute path or relative path (relative to the config file location)
    pub path: Option<String>,

    #[serde(skip_serializing)]
    pub data_path: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            kind: Databases::Sled,
            path: None,
            temporary: Some(false),
            uri: None,
            data_path: None,
        }
    }
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
    #[error("Block not found")]
    InvalidTransactionData,
    #[error("Invalid txdata")]
    BlockNotFound,
    #[error("db insert failed at")]
    DBInsertFailed,
    #[error("An error occured: {0}")]
    Specific(String),

    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),

    // Backend Specific Erros
    #[cfg(feature = "sql")]
    #[error(transparent)]
    SeaORM(#[from] entity::sea_orm::DbErr),

    // Backend Specific Erros
    #[cfg(feature = "sled")]
    #[error(transparent)]
    Sled(#[from] sled::sled::Error),
}

pub async fn new(cfg: &DatabaseConfig) -> Result<Box<dyn Database>, DatabaseError> {
    #[allow(clippy::needless_late_init)]
    let db: Box<dyn Database>;
    match cfg.kind {
        #[cfg(feature = "backend-sqlite")]
        Databases::SQLite => {
            // let database = sql::Sql::connect_sqlite(cfg).await.map_err(|e| DatabaseError::Specific(e.to_string()))?;
            let database = sql::Sql::connect_mock().await.map_err(|e| DatabaseError::Specific(e.to_string()))?;
            db = Box::new(database);
        }
        #[cfg(feature = "backend-sled")]
        Databases::Sled => {
            db = Box::new(sled::SledDB::new(cfg).map_err(|e| DatabaseError::Specific(e.to_string()))?);
        }
        #[allow(unreachable_patterns)]
        _ => return Err(DatabaseError::InvalidKind),
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
    async fn get_block_by_id(&self, block_id: api::BlockID) -> Result<api::SignedBlock, DatabaseError>;
    async fn get_block_by_height(
        &self,
        account_id: api::AccountID,
        block_height: &u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError>;
    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError>;

    /// Finds the latest block for a given address
    ///
    /// Only includes confirmed blocks
    async fn get_latest_block_by_account(&self, acc_id: api::AccountID) -> Result<api::SignedBlock, DatabaseError>;

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
    ) -> Result<Option<api::SignedBlock>, DatabaseError>;

    // get_blocks returns a list of blocks, NOTE: if account_id is not specified, this is not atomic! block order might not be correct and never can be, since consensus is seperate for each account chain.
    async fn get_blocks(
        &self,
        newest: bool,
        limit: u32,
        offset: u32,
        account_id: Option<AccountID>,
    ) -> Result<Vec<api::SignedBlock>, DatabaseError>;

    // get_account_delegate finds out if an account is delegating their power to someone else
    async fn get_account_delegate(&self, account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError>;

    // Finds who is delegating power to an account
    async fn get_delegates_by_account(&self, account_id: api::AccountID)
        -> Result<Vec<api::AccountID>, DatabaseError>;

    // Adds a new block to the database
    async fn add_block(&mut self, block: api::SignedBlock) -> Result<(), DatabaseError>;

    // Get the transaction id claiming a send transaction
    async fn get_send_recipient(
        &self,
        send_transaction_id: api::TransactionID,
    ) -> Result<Option<api::TransactionID>, DatabaseError>;
}
