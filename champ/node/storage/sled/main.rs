use std::convert::TryInto;

use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use pog_proto::api::{self, AccountID, BlockID};
use prost::Message;
use sled::Transactional;

#[derive(Debug)]
pub struct SledDB {
    // db: sled::Db,
    // pending_blocks: sled::Tree,
    blocks: sled::Tree,
    accounts: sled::Tree,
    transactions: sled::Tree,
    claims: sled::Tree,
    // meta: sled::Tree,
}

impl SledDB {
    pub fn new(cfg: &DatabaseConfig) -> Result<Self> {
        let mut sled_cfg = sled::Config::default();

        if cfg.temporary.unwrap_or(false) {
            sled_cfg = sled_cfg.temporary(true);
        } else {
            sled_cfg = sled_cfg.path(cfg.data_path.as_ref().expect("sled db path needs to be specified"));
        }

        let db: sled::Db = sled_cfg.open()?;
        // let pending_blocks = db.open_tree("pending_blocks")?;
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
        // key: account_id + "_last_blk"
        // val: latest block id
        //
        // key: account_id + "_rep"
        // val: representative account_id

        // claims provides some convenient pointers to data relevant to claim transactions
        let claims = db.open_tree("claims")?;
        // claims contain
        // key: send_transaction_id
        // val: recieve_transaction_id

        let blocks = db.open_tree("blocks")?;
        // blocks contain:
        //
        // key: "by_id_" + block_id
        // val: block proto
        //
        // key: "by_acc_" + account_id + "_" + block_height
        // val: block_id

        // transactions provides a list of transactions as a fast way to get transactions by their transaction id
        let transactions = db.open_tree("transactions")?;
        // transactions contain:
        //
        // key: "by_id_" + transaction_id
        // val: transaction proto
        //
        // key: "blk_by_id_" + transaction_id
        // val: block_id
        //
        // key: "by_blk_id_" + block_id + "block_index"
        // val: transaction proto

        // let meta = db.open_tree("meta")?;

        Ok(Self {
            // db,
            // pending_blocks,
            blocks,
            accounts,
            transactions,
            claims,
            // meta,
        })
    }
}

#[async_trait]
impl Database for SledDB {
    async fn get_send_recipient(&self, tx: api::TransactionID) -> Result<Option<api::TransactionID>, DatabaseError> {
        let claim = self.claims.get(tx).map_err(|e| DatabaseError::Specific(e.to_string()))?;

        match claim {
            Some(tx_id) => {
                let id: api::TransactionID = tx_id
                    .to_vec()
                    .try_into()
                    .map_err(|_| DatabaseError::Specific("invalid transaction id".to_string()))?;
                Ok(Some(id))
            }
            None => Ok(None),
        }
    }

    async fn get_block_by_id(&self, block_id: api::BlockID) -> Result<api::SignedBlock, DatabaseError> {
        let mut block_key = b"by_id_".to_vec();
        block_key.append(&mut block_id.to_vec());

        let block = self
            .blocks
            .get(block_key)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?
            .ok_or(DatabaseError::BlockNotFound)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?;

        api::SignedBlock::decode(&*block.to_vec()).map_err(|e| DatabaseError::Specific(e.to_string()))
    }

    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError> {
        let mut transaction_key = b"by_id_".to_vec();
        transaction_key.append(&mut transaction_id.to_vec());

        let transaction = self
            .transactions
            .get(transaction_key)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?
            .ok_or(DatabaseError::BlockNotFound)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?;

        api::Transaction::decode(&*transaction.to_vec()).map_err(|e| DatabaseError::Specific(e.to_string()))
    }

    async fn get_latest_block_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<api::SignedBlock, DatabaseError> {
        let mut last_block_key = account_id.to_vec();
        last_block_key.append(&mut b"_last_blk".to_vec());

        let latest_block_id: BlockID = self
            .accounts
            .get(last_block_key)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?
            .ok_or(DatabaseError::BlockNotFound)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?
            .to_vec()
            .try_into()
            .map_err(|_| DatabaseError::BlockNotFound)?;

        let mut block_key = b"by_id_".to_vec();
        block_key.append(&mut latest_block_id.to_vec());

        let block = self
            .blocks
            .get(block_key)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?
            .ok_or(DatabaseError::BlockNotFound)
            .map_err(|e| DatabaseError::Specific(e.to_string()))?;

        api::SignedBlock::decode(&*block.to_vec()).map_err(|e| DatabaseError::Specific(e.to_string()))
    }

    async fn add_block(&mut self, block: api::SignedBlock) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().expect("block should have valid data");
        let block_id = block.get_id().map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let account_id = encoding::account::generate_account_address(block.public_key.clone())
            .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;

        let res: sled::transaction::TransactionResult<()> =
            (&self.accounts, &self.blocks, &self.transactions, &self.claims).transaction(
                |(accounts, blocks, transactions, claims)| {
                    let mut block_key = b"by_id_".to_vec();
                    block_key.append(&mut block_id.to_vec());

                    let mut block_by_acc_key = b"by_acc_".to_vec();
                    block_by_acc_key.append(&mut account_id.to_vec());
                    block_by_acc_key.append(&mut b"_".to_vec());
                    block_by_acc_key.append(&mut block_data.height.to_be_bytes().to_vec());

                    // Set as latest block
                    let mut account_key = account_id.to_vec();
                    account_key.append(&mut b"_last_blk".to_vec());
                    accounts.insert(account_key, &block_id.clone())?;

                    // Add Block
                    blocks.insert(block_key, block.encode_to_vec())?;
                    blocks.insert(block_by_acc_key, block_id.to_vec())?;

                    // Add Block Transactions
                    let mut batch = sled::Batch::default();
                    for (i, tx) in block_data.transactions.iter().enumerate() {
                        let tx_data = tx.data.clone().expect("transaction should have valid data");

                        let transaction_id = match tx.get_id(block_id) {
                            Ok(x) => x,
                            Err(_) => return sled::transaction::abort(()),
                        };

                        match tx_data {
                            // Set representative
                            api::transaction::Data::TxDelegate(tx) => {
                                let mut account_rep_key = b"rep_".to_vec();
                                account_rep_key.append(&mut account_id.to_vec());
                                accounts.insert(account_rep_key, tx.representative)?;
                            }
                            // Set claims
                            api::transaction::Data::TxClaim(tx) => {
                                claims.insert(tx.send_transaction_id, transaction_id.to_vec())?;
                            }
                            _ => {}
                        };

                        let tx = tx.encode_to_vec();

                        // "by_id_" + transaction_id
                        let mut tx_key = b"by_id_".to_vec();
                        tx_key.append(&mut transaction_id.into());
                        batch.insert(tx_key, tx.clone());

                        // "by_id_" + transaction_id + "_blk"
                        let mut tx_key = b"blk_by_id_".to_vec();
                        tx_key.append(&mut transaction_id.into());
                        batch.insert(tx_key, &block_id);

                        // "by_blk_id_" + block_id + "block_index"
                        let mut tx_key = b"by_blk_id_".to_vec();
                        tx_key.append(&mut block_id.into());
                        tx_key.append(&mut i.to_be_bytes().into());
                        batch.insert(tx_key, tx);
                    }
                    transactions.apply_batch(&batch)?;

                    Ok(())
                },
            );

        res.map_err(|_| DatabaseError::DBInsertFailed(line!()))
    }

    async fn get_block_by_height(
        &self,
        account_id: api::AccountID,
        block_height: &u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        let mut block_key = b"by_acc_".to_vec();
        block_key.append(&mut account_id.into());
        block_key.append(&mut b"_".to_vec());
        block_key.append(&mut block_height.to_be_bytes().into());

        let block_id = self.blocks.get(block_key).map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let block_id = match block_id {
            Some(block_id) => block_id,
            None => return Ok(None),
        };

        let mut block_key = b"by_id_".to_vec();
        block_key.append(&mut block_id.to_vec());

        let block = self.blocks.get(block_key).map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let block = match block {
            Some(block) => block,
            None => return Ok(None),
        };

        api::SignedBlock::decode(&*block.to_vec()).map(Some).map_err(|e| DatabaseError::Specific(e.to_string()))
    }

    async fn get_account_delegate(&self, account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError> {
        let mut last_block_key = account_id.to_vec();
        last_block_key.append(&mut b"_rep".to_vec());

        let delegate_id = self.accounts.get(last_block_key).map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let delegate_id: AccountID = match delegate_id {
            Some(delegate) => {
                delegate.to_vec().try_into().map_err(|_| DatabaseError::Specific("invalid account id".to_string()))?
            }
            None => return Ok(None),
        };

        Ok(Some(delegate_id))
    }

    // TODO: THIS IS NOT OPTIMIZED FOR PERFORMANCE AND BASED ON
    // OUR MOCK DATABASE CODE! BEWARE! HERE BE DRAGONS!
    async fn get_delegates_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<Vec<api::AccountID>, DatabaseError> {
        let mut delegated_accounts = std::collections::HashSet::new();

        let stuff = self.transactions.scan_prefix(b"blk_by_id_").rev().filter_map(|res| {
            let (key, value) = res.ok()?;
            let tx = api::Transaction::decode(&*value.to_vec()).ok()?;
            let blk_id: BlockID = key[10..].try_into().ok()?;
            Some((blk_id, tx))
        });

        for (blk_id, tx) in stuff {
            let blk = self.get_block_by_id(blk_id).await?;
            let tx_acc = encoding::account::generate_account_address(blk.public_key.clone())
                .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;

            if let Some(api::transaction::Data::TxDelegate(delegate_tx)) = &tx.data {
                if delegate_tx.representative == account_id {
                    // only the latest transaction counts per account
                    if delegated_accounts.contains(&tx_acc) {
                        continue;
                    }
                    delegated_accounts.insert(tx_acc);
                }
            }
        }

        Ok(delegated_accounts.into_iter().collect())
    }

    // TODO: THIS IS NOT OPTIMIZED FOR PERFORMANCE AND BASED ON
    // OUR MOCK DATABASE CODE! BEWARE! HERE BE DRAGONS!
    async fn get_latest_block_by_account_before(
        &self,
        account_id: api::AccountID,
        unix_from: u64,
        unix_limit: u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        self.blocks
            .scan_prefix(b"by_id_")
            // reverse to make it faster for newer blocks
            .rev()
            .find_map(|res| {
                let (_, value) = res.ok()?;
                let block = api::SignedBlock::decode(&*value.to_vec()).ok()?;
                let block_account_id = encoding::account::generate_account_address(block.public_key.clone()).ok()?;

                if block_account_id == account_id {
                    if block.timestamp < unix_limit {
                        return Some(None);
                    }
                    if block.timestamp < unix_from {
                        return Some(Some(block));
                    }
                }
                None
            })
            .ok_or(DatabaseError::Unknown)
    }
}
