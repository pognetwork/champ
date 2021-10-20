use crate::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api;

use sled;

#[derive(Debug)]
pub struct SledDB {
    db: sled::Db,
    pending_blocks: sled::Tree,
    blocks: sled::Tree,
    accounts: sled::Tree,
    transactions: sled::Tree,
    meta: sled::Tree,
}

impl SledDB {
    pub fn new(cfg: &DatabaseConfig) -> Result<Self> {
        let db: sled::Db = sled::open(cfg.data_path.as_ref().expect("sled db path needs to be specified"))?;

        let pending_blocks = db.open_tree("pending_blocks")?;
        // pending_blocks contain:
        //
        // key: generate_id
        // val: block proto
        // // this is used after e.g a server crash to recover the pending log
        // // these are atomically moved to blocks once accepted

        // accounts provides some convenient pointers to data relevant to an account
        let accounts = db.open_tree("accounts")?;
        // accounts contain:
        //
        // key: account_id + "_lastblk"
        // val: latest block id

        let blocks = db.open_tree("blocks")?;
        // blocks contain:
        //
        // key: "blk_" + account_id + "_" + block_height
        // val: block proto

        // transactions provides a list of transactions as a fast way to get transactions by their transaction id
        let transactions = db.open_tree("transactions")?;
        // transactions contain:
        //
        // key: "byid_" + transaction_id
        // val: transaction proto
        //
        // key: "byblk_" + block_id + "block_index"
        // val: transaction proto

        let meta = db.open_tree("meta")?;

        Ok(Self {
            db,
            pending_blocks,
            blocks,
            accounts,
            transactions,
            meta,
        })
    }
}

#[async_trait]
impl Database for SledDB {
    async fn get_block_by_id(&self, _block_id: api::BlockID) -> Result<&api::Block, DatabaseError> {
        unimplemented!("")
    }

    async fn get_transaction_by_id(
        &self,
        _transaction_id: api::TransactionID,
    ) -> Result<&api::Transaction, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_latest_block_by_account(&self, _account_id: api::AccountID) -> Result<&api::Block, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn add_block(&mut self, _block: api::Block) -> Result<(), DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_block_by_height(
        &self,
        _account_id: api::AccountID,
        _block_height: &u64,
    ) -> Result<Option<&api::Block>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_account_delegate(&self, _account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_delegates_by_account(
        &self,
        _account_id: api::AccountID,
    ) -> Result<Vec<api::AccountID>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_latest_block_by_account_before(
        &self,
        _account_id: api::AccountID,
        _unix_from: u64,
        _unix_limit: u64,
    ) -> Result<Option<&api::Block>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }
}
