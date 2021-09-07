use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api;

use crate::{Database, DatabaseConfig, DatabaseError};

type TransactionIDs = Vec<String>;
type BlockIDs = Vec<String>;

pub struct MockDB {
    blocks: HashMap<String, api::Block>,
    accounts: HashMap<String, (api::PublicAccount, BlockIDs, TransactionIDs)>,
    transactions: HashMap<String, api::Transaction>,
}

impl MockDB {
    pub fn new() -> Self {
        let blocks = HashMap::new();
        let accounts = HashMap::new();
        let transactions = HashMap::new();

        Self {
            blocks,
            accounts,
            transactions,
        }
    }
}

// pub fn create_block() {}

#[async_trait]
impl Database for MockDB {
    async fn init(&mut self, _: &DatabaseConfig) -> Result<()> {
        Ok(())
    }

    async fn get_block_by_id(&self, block_id: &str) -> Result<&api::Block, DatabaseError> {
        self.blocks.get(block_id).ok_or(DatabaseError::Unknown)
    }

    async fn get_transaction_by_id(
        &self,
        transaction_id: &str,
    ) -> Result<&api::Transaction, DatabaseError> {
        self.transactions
            .get(transaction_id)
            .ok_or(DatabaseError::Unknown)
    }

    async fn get_latest_block_by_account(
        &self,
        account_id: &str,
    ) -> Result<&api::Block, DatabaseError> {
        let (_account, blocks, _txs) = self
            .accounts
            .get(account_id)
            .ok_or(DatabaseError::Unknown)?;

        let last_block_id = blocks.last().clone().ok_or(DatabaseError::Unknown)?;
        self.get_block_by_id(&last_block_id).await
    }

    async fn get_transactions_by_account(
        &self,
        account_id: &str,
    ) -> Result<&api::PublicAccount, DatabaseError> {
        let (account, _blocks, _txs) = self
            .accounts
            .get(account_id)
            .ok_or(DatabaseError::Unknown)?;
        Ok(account)
    }

    async fn add_block(&mut self, block: api::Block) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().ok_or(DatabaseError::Unknown)?;
        let account_hash = hex::encode(&block_data.address.clone());
        let block_hash = hex::encode(&block.hash.clone());

        let (_account, blocks, transactions) = self
            .accounts
            .get_mut(&account_hash)
            .ok_or(DatabaseError::Unknown)?;

        self.blocks
            .insert(block_hash.clone(), block.clone())
            .ok_or(DatabaseError::Unknown)?;

        blocks.push(block_hash.clone());

        for tx in block_data.transactions {
            let tx_id = crypto::hash::sha3([tx.hash.clone(), block.hash.clone()].concat());
            let tx_id_str = hex::encode(&tx_id);

            transactions.push(tx_id_str.clone());
            self.transactions
                .insert(tx_id_str, tx)
                .ok_or(DatabaseError::Unknown)?;
        }
        Ok(())
    }
}