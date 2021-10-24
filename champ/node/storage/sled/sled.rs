use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api;
use prost::Message;
use sled::Transactional;
use sled::{self};

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
        // key: "byblk_" + block_id
        // key: "byacc_" + account_id + "_" + block_height
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
    // impl SledDB {
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

    async fn add_block(&mut self, block: api::Block) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().ok_or(DatabaseError::DataNotFound)?;
        let block_id = block.get_id().map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let account_id = encoding::account::generate_account_address(block.public_key.clone())
            .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;

        let res: sled::transaction::TransactionResult<()> = (&self.accounts, &self.blocks, &self.transactions)
            .transaction(|(accounts, blocks, transactions)| {
                let mut block_key = b"byblk_".to_vec();
                block_key.append(&mut block_id.to_vec());

                // Add new accounts
                if block_data.height == 0 {
                    let mut account_key = b"lastblk_".to_vec();
                    account_key.append(&mut account_id.to_vec());
                    accounts.insert(account_key, &block_id.clone())?;
                };

                // Add Block
                blocks.insert(block_key, block.encode_to_vec())?;

                // Add Block Transactions
                let mut batch = sled::Batch::default();
                for (i, tx) in block_data.transactions.iter().enumerate() {
                    let transaction_id = match tx.get_id(block_id) {
                        Ok(x) => x,
                        Err(_) => return sled::transaction::abort(()),
                    };

                    let tx = tx.encode_to_vec();

                    // "byid_" + transaction_id
                    let mut tx_key = b"byid_".to_vec();
                    tx_key.append(&mut transaction_id.into());
                    batch.insert(tx_key, tx.clone());

                    // "byblk_" + block_id + "block_index"
                    let mut tx_key = b"byblk_".to_vec();
                    tx_key.append(&mut block_id.into());
                    tx_key.append(&mut i.to_be_bytes().into());
                    batch.insert(tx_key, tx);
                }
                transactions.apply_batch(&batch)?;

                Ok(())
            });

        res.map_err(|_| DatabaseError::DBInsertFailed(line!()))
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
