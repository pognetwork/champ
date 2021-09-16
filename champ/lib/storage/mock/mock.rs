use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api::{self, transaction::Data::TxDelegate};

use crate::{Database, DatabaseConfig, DatabaseError};

type TransactionIDs = Vec<String>;
type BlockIDs = Vec<String>;
type AccountID = String;

#[derive(Default, Debug)]
pub struct MockDB {
    blocks: BTreeMap<String, api::Block>,
    accounts: HashMap<String, (api::PublicAccount, BlockIDs, TransactionIDs)>,
    transactions: BTreeMap<String, (api::Transaction, AccountID)>,
}

impl MockDB {
    pub fn new() -> Self {
        let blocks = BTreeMap::new();
        let accounts = HashMap::new();
        let transactions = BTreeMap::new();

        Self {
            blocks,
            accounts,
            transactions,
        }
    }
}

#[async_trait]
impl Database for MockDB {
    async fn init(&mut self, _: &DatabaseConfig) -> Result<()> {
        Ok(())
    }

    async fn get_block_by_id(&self, block_id: &str) -> Result<&api::Block, DatabaseError> {
        self.blocks.get(block_id).ok_or(DatabaseError::Unknown)
    }

    async fn get_transaction_by_id(&self, transaction_id: &str) -> Result<&api::Transaction, DatabaseError> {
        self.transactions
            .get(transaction_id)
            .ok_or(DatabaseError::Unknown)
            .map(|tx| &tx.0)
    }

    async fn get_latest_block_by_account(&self, account_id: &str) -> Result<&api::Block, DatabaseError> {
        let (_account, blocks, _txs) = self.accounts.get(account_id).ok_or(DatabaseError::Unknown)?;

        let last_block_id = blocks.last().ok_or(DatabaseError::NoLastBlock)?;
        self.get_block_by_id(last_block_id).await
    }

    async fn add_block(&mut self, block: api::Block) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().ok_or(DatabaseError::Unknown)?;
        let account_hash = hex::encode(&block_data.address);
        let block_hash = hex::encode(&block.hash);

        let (_account, account_blocks, account_transactions) =
            self.accounts.get_mut(&account_hash).ok_or(DatabaseError::Unknown)?;

        self.blocks
            .insert(block_hash.clone(), block.clone())
            .ok_or(DatabaseError::Unknown)?;

        account_blocks.push(block_hash);

        for tx in block_data.transactions {
            let tx_id = crypto::hash::sha3([tx.hash.clone(), block.hash.clone()].concat());
            let tx_id_str = hex::encode(&tx_id);

            account_transactions.push(tx_id_str.clone());
            self.transactions
                .insert(tx_id_str, (tx, account_hash.clone()))
                .ok_or(DatabaseError::Unknown)?;
        }
        Ok(())
    }

    async fn get_block_by_height(&self, account_id: &str, block_height: &u64) -> Result<&api::Block, DatabaseError> {
        self.blocks
            .iter()
            // reverse to make it faster for newer blocks
            .rev()
            .find_map(|b| {
                let (_,block) = b;
                if matches!(block.to_owned().data, Some(block_data) if block_data.address == account_id && &block_data.height == block_height) {
                    Some(block)
                } else {
                    None
                }
            })
            .ok_or(DatabaseError::Unknown)
    }

    async fn get_account_delegate(&self, account_id: &str) -> Result<Option<String>, DatabaseError> {
        let delegate = self
            .transactions
            .iter()
            // reverse since only the newest transaction counts
            .rev()
            .find_map(|(_, (tx, tx_acc))| {
                if let Some(TxDelegate(delegate_tx)) = &tx.data {
                    if tx_acc == account_id {
                        return Some(Some(delegate_tx.representative.clone()));
                    }
                }
                None
            })
            .ok_or(DatabaseError::Unknown)?
            .ok_or(DatabaseError::Unknown)?;

        let delegate_str = hex::encode(delegate);
        if delegate_str.is_empty() {
            Ok(None)
        } else {
            Ok(Some(delegate_str))
        }
    }

    async fn get_delegates_by_account(&self, account_id: &str) -> Result<Vec<String>, DatabaseError> {
        let mut delegated_accounts = HashSet::new();
        let account_hex = hex::decode(account_id).map_err(|_| DatabaseError::Unknown)?;

        self.transactions.iter().rev().for_each(|(_, (tx, tx_acc))| {
            if let Some(TxDelegate(delegate_tx)) = &tx.data {
                if delegate_tx.representative == account_hex {
                    // only the latest transaction counts per account
                    if delegated_accounts.contains(tx_acc) {
                        return;
                    }
                    delegated_accounts.insert(tx_acc.clone());
                }
            }
        });

        Ok(delegated_accounts.into_iter().collect())
    }

    async fn get_latest_block_by_account_before(
        &self,
        account_id: &str,
        unix_from: u64,
        unix_limit: u64,
    ) -> Result<Option<&api::Block>, DatabaseError> {
        self.blocks
            .iter()
            // reverse to make it faster for newer blocks
            .rev()
            .find_map(|b| {
                let block = b.1.to_owned();
                if matches!(block.data, Some(block_data) if block_data.address == account_id) {
                    if block.timestamp < unix_limit {
                        return Some(None);
                    }
                    if block.timestamp < unix_from {
                        return Some(Some(b.1));
                    }
                }
                None
            })
            .ok_or(DatabaseError::Unknown)
    }
}
