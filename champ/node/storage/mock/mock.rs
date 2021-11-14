use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api::{self, transaction::Data::TxDelegate};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryInto,
};

#[derive(Default, Debug)]
pub struct MockDB {
    blocks: BTreeMap<api::BlockID, (api::SignedBlock, api::AccountID)>,
    accounts: HashMap<api::AccountID, (api::Account, Vec<api::BlockID>, Vec<api::TransactionID>)>,
    transactions: BTreeMap<api::TransactionID, (api::Transaction, api::AccountID)>,
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

    async fn get_block_by_id(&self, block_id: api::BlockID) -> Result<api::SignedBlock, DatabaseError> {
        let (block, _) = self.blocks.get(&block_id).ok_or(DatabaseError::Unknown)?;
        Ok(block.to_owned())
    }

    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError> {
        self.transactions.get(&transaction_id).ok_or(DatabaseError::Unknown).map(|(tx, _)| tx.to_owned())
    }

    async fn get_latest_block_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<api::SignedBlock, DatabaseError> {
        let (_account, blocks, _txs) = self.accounts.get(&account_id).ok_or(DatabaseError::Unknown)?;

        let last_block_id = blocks.last().ok_or(DatabaseError::NoLastBlock)?;
        self.get_block_by_id(*last_block_id).await
    }

    async fn add_block(&mut self, block: api::SignedBlock) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().ok_or(DatabaseError::DataNotFound)?;

        let account_id = encoding::account::generate_account_address(block.public_key.clone())
            .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;
        let block_hash = block.get_id().map_err(|_| DatabaseError::GetIDFailed)?;

        if block_data.height == 0 {
            // create new account
            if self.accounts.get(&account_id).is_some() {
                return Err(DatabaseError::Specific("account already exists but block height == 0".to_string()));
            }
            self.accounts.insert(
                account_id,
                (
                    api::Account {
                        r#type: 0,
                        address: account_id.to_vec(),
                        voting_power: 0,
                        public_key: vec![],
                        private_key: vec![],
                    },
                    vec![],
                    vec![],
                ),
            );
        }

        let (_account, account_blocks, account_transactions) = self
            .accounts
            .get_mut(&account_id)
            .ok_or_else(|| DatabaseError::Specific("account could not be fetched".to_string()))?;

        if self.blocks.get(&block_hash).is_some() {
            return Err(DatabaseError::DBInsertFailed(line!()));
        };
        self.blocks.insert(block_hash, (block, account_id));

        account_blocks.push(block_hash);

        for tx in block_data.transactions {
            let tx_id = tx.get_id(block_hash).map_err(|_| DatabaseError::GetIDFailed)?;

            if self.transactions.get(&tx_id).is_some() {
                return Err(DatabaseError::DBInsertFailed(line!()));
            }
            account_transactions.push(tx_id);
            self.transactions.insert(tx_id, (tx, account_id));
        }
        Ok(())
    }

    async fn get_block_by_height(
        &self,
        account_id: api::AccountID,
        block_height: &u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        self.blocks
            .iter()
            // reverse to make it faster for newer blocks
            .rev()
            .find_map(|(_, (block, account))| { //TODO: check this returns none if nothing was found
                if matches!(block.to_owned().data, Some(block_data) if *account == account_id && &block_data.height == block_height) {
                    Some(Some(block.to_owned()))
                } else {
                    None
                }
            })
            .ok_or(DatabaseError::Unknown)
    }

    async fn get_account_delegate(&self, account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError> {
        let delegate = self
            .transactions
            .iter()
            // reverse since only the newest transaction counts
            .rev()
            .find_map(|(_, (tx, tx_acc))| {
                if let Some(TxDelegate(delegate_tx)) = &tx.data {
                    if *tx_acc == account_id {
                        return Some(Some(delegate_tx.representative.clone()));
                    }
                }
                None
            })
            .ok_or(DatabaseError::Unknown)?
            .ok_or(DatabaseError::Unknown)?;

        let d: api::AccountID = match delegate.try_into() {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };
        Ok(Some(d))
    }

    async fn get_delegates_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<Vec<api::AccountID>, DatabaseError> {
        let mut delegated_accounts = HashSet::new();

        self.transactions.iter().rev().for_each(|(_, (tx, tx_acc))| {
            if let Some(TxDelegate(delegate_tx)) = &tx.data {
                if delegate_tx.representative == account_id {
                    // only the latest transaction counts per account
                    if delegated_accounts.contains(tx_acc) {
                        return;
                    }
                    delegated_accounts.insert(*tx_acc);
                }
            }
        });

        Ok(delegated_accounts.into_iter().collect())
    }

    async fn get_latest_block_by_account_before(
        &self,
        account_id: api::AccountID,
        unix_from: u64,
        unix_limit: u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        self.blocks
            .iter()
            // reverse to make it faster for newer blocks
            .rev()
            .find_map(|(_, (block, block_address))| {
                if *block_address == account_id {
                    if block.timestamp < unix_limit {
                        return Some(None);
                    }
                    if block.timestamp < unix_from {
                        return Some(Some(block.to_owned()));
                    }
                }
                None
            })
            .ok_or(DatabaseError::Unknown)
    }
}
