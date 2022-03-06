//! SQL Storage Backend for pog.network champ !UNSTABLE!
//!
//! This backend is currently untested and only supports the in-memory sqlite driver

use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use entity::sea_orm::{
    self, sea_query::TableCreateStatement, ConnectOptions, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use entity::unix_to_datetime;
use pog_proto::api;

use entity::account::{self, Entity as Account};
use entity::block::{self, Entity as Block};
use entity::pending_block::Entity as PendingBlock;
use entity::transaction::{self, Entity as Transaction};
use entity::tx_claim::{self, Entity as TxClaim};
use prost::Message;

#[derive(Debug)]
pub struct Sql {
    db: DatabaseConnection,
}

impl Sql {
    #[cfg(feature = "backend-sqlite")]
    pub async fn connect_mock() -> Result<Sql> {
        let opt = ConnectOptions::new("sqlite::memory:".to_string());

        let db = sea_orm::Database::connect(opt).await?;
        let sql = Sql {
            db,
        };

        sql.setup_schema(sea_orm::DatabaseBackend::Sqlite).await?;

        Ok(sql)
    }

    #[allow(dead_code)]
    pub async fn connect_sqlite(_cfg: &DatabaseConfig) -> Result<Sql> {
        unimplemented!("");
    }

    // not required after we've setup migrations
    pub async fn setup_schema(&self, backend: DbBackend) -> Result<()> {
        let schema = Schema::new(backend);

        let account: TableCreateStatement = schema.create_table_from_entity(Account);
        let block: TableCreateStatement = schema.create_table_from_entity(Block);
        let pending_block: TableCreateStatement = schema.create_table_from_entity(PendingBlock);
        let transaction: TableCreateStatement = schema.create_table_from_entity(Transaction);
        let tx_claim: TableCreateStatement = schema.create_table_from_entity(TxClaim);

        self.db.execute(self.db.get_database_backend().build(&account)).await?;
        self.db.execute(self.db.get_database_backend().build(&block)).await?;
        self.db.execute(self.db.get_database_backend().build(&pending_block)).await?;
        self.db.execute(self.db.get_database_backend().build(&transaction)).await?;
        self.db.execute(self.db.get_database_backend().build(&tx_claim)).await?;

        Ok(())
    }
}

#[async_trait]
impl Database for Sql {
    async fn get_block_by_id(&self, block_id: api::BlockID) -> Result<api::SignedBlock, DatabaseError> {
        let block = Block::find_by_id(block_id.into()).one(&self.db).await?.ok_or(DatabaseError::BlockNotFound)?;
        Ok(block.try_into()?)
    }

    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError> {
        let transaction =
            Transaction::find_by_id(transaction_id.into()).one(&self.db).await?.ok_or(DatabaseError::BlockNotFound)?;

        api::Transaction::decode(&*transaction.data).map_err(DatabaseError::DecodeError)
    }

    async fn get_latest_block_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<api::SignedBlock, DatabaseError> {
        let (_, block) = Account::find_by_id(account_id.into())
            .find_also_related(Block)
            .select_only()
            .column(account::Column::LatestBlock)
            .column(block::Column::Data)
            .one(&self.db)
            .await?
            .ok_or(DatabaseError::BlockNotFound)?;

        let block = block.ok_or(DatabaseError::BlockNotFound)?;
        Ok(block.try_into()?)
    }

    async fn add_block(&mut self, block: api::SignedBlock) -> Result<(), DatabaseError> {
        let block_id = block.get_id();
        let account_id = encoding::account::generate_account_address(block.header.public_key.clone())
            .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;

        let txn = self.db.begin().await?;

        let new_block = block::ActiveModel {
            height: Set(block.data.height),
            account_id_v1: Set(account_id.into()),
            balance: Set(block.data.balance),
            block_id: Set(block_id.into()),
            data: Set(block.data_raw),
            public_key: Set(block.header.public_key.clone()),
            signature: Set(block.header.signature),
            timestamp: Set(unix_to_datetime(block.header.timestamp)),
            version: Set(block::BlockVersion::V1),
        };

        let mut account: account::ActiveModel =
            match Account::find_by_id(block.header.public_key.clone()).one(&txn).await? {
                Some(account) => account.into(),
                None => {
                    let new_acc = account::ActiveModel {
                        public_key: Set(block.header.public_key),
                        account_id_v1: Set(account_id.into()),
                        latest_block: Set(block_id.into()),
                        delegate: Set(None),
                    };
                    new_acc.insert(&txn).await?.into()
                }
            };

        let mut transactions: Vec<transaction::ActiveModel> = vec![];
        let mut new_delegate: Option<Vec<u8>> = None;
        for (i, tx) in block.data.transactions.iter().enumerate() {
            let tx_data = tx.data.clone().ok_or(DatabaseError::InvalidTransactionData)?;
            let transaction_id =
                api::Transaction::get_id(block_id, i as u32).map_err(|_| DatabaseError::InvalidTransactionData)?;

            let mut claims: Vec<tx_claim::ActiveModel> = vec![];
            let tx_type: transaction::TxType;
            match tx_data {
                // Set representative
                api::transaction::Data::TxDelegate(tx) => {
                    let mut account_rep_key = b"rep_".to_vec();
                    account_rep_key.append(&mut account_id.to_vec());
                    new_delegate = Some(tx.representative);
                    tx_type = transaction::TxType::TxDelegate;
                }
                // Set claims
                api::transaction::Data::TxClaim(tx) => {
                    claims.push(tx_claim::ActiveModel {
                        claim_tx_id: Set(transaction_id.to_vec()),
                        send_tx_id: Set(tx.send_transaction_id),
                    });
                    tx_type = transaction::TxType::TxClaim;
                }
                api::transaction::Data::TxOpen(_) => tx_type = transaction::TxType::TxOpen,
                api::transaction::Data::TxSend(_) => tx_type = transaction::TxType::TxSend,
            };

            transactions.push(transaction::ActiveModel {
                block_id: Set(block_id.into()),
                transaction_id: Set(transaction_id.to_vec()),
                data: Set(tx.encode_to_vec()),
                tx_type: Set(tx_type),
            })
        }

        Transaction::insert_many(transactions).exec(&txn).await?;
        Block::insert(new_block).exec(&txn).await?;

        if new_delegate.is_some() {
            account.delegate = Set(new_delegate);
            account.update(&txn).await?;
        }

        txn.commit().await?;
        Ok(())
    }

    async fn get_block_by_height(
        &self,
        account_id: api::AccountID,
        block_height: &u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        let block = Block::find()
            .filter(block::Column::Height.eq(*block_height))
            .filter(block::Column::AccountIdV1.eq(account_id.to_vec()))
            .select_only()
            .column(block::Column::Data)
            .one(&self.db)
            .await?;

        match block {
            Some(block) => Ok(Some(block.try_into()?)),

            None => Ok(None),
        }
    }

    async fn get_account_delegate(&self, account_id: api::AccountID) -> Result<Option<api::AccountID>, DatabaseError> {
        let account = Account::find_by_id(account_id.into())
            .select_only()
            .column(account::Column::Delegate)
            .one(&self.db)
            .await?
            .ok_or(DatabaseError::BlockNotFound)?;

        match account.delegate {
            Some(id) => Ok(Some(
                api::AccountID::try_from(id).map_err(|_| DatabaseError::Specific("invalid account id".to_string()))?,
            )),
            None => Ok(None),
        }
    }

    async fn get_delegates_by_account(
        &self,
        account_id: api::AccountID,
    ) -> Result<Vec<api::AccountID>, DatabaseError> {
        let accounts = Account::find()
            .filter(account::Column::Delegate.eq(account_id.to_vec()))
            .select_only()
            .column(account::Column::AccountIdV1)
            .all(&self.db)
            .await?;

        Ok(accounts
            .iter()
            .filter_map(|a| {
                api::AccountID::try_from(a.account_id_v1.clone())
                    .map_err(|_| DatabaseError::Specific("invalid account id".to_string()))
                    .ok()
            })
            .collect::<Vec<api::AccountID>>())
    }

    async fn get_latest_block_by_account_before(
        &self,
        account_id: api::AccountID,
        unix_from: u64,
        unix_limit: u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        let from = unix_to_datetime(unix_from);
        let to = unix_to_datetime(unix_limit);

        let block = Block::find()
            .filter(block::Column::AccountIdV1.eq(account_id.to_vec()))
            .filter(block::Column::Timestamp.between(from, to))
            .order_by_asc(block::Column::Timestamp)
            .column(block::Column::Data)
            .one(&self.db)
            .await?;

        Ok(match block {
            Some(block) => {
                let block: api::SignedBlock = block.try_into()?;
                Some(block)
            }
            None => None,
        })
    }

    async fn get_send_recipient(
        &self,
        send_transaction_id: api::TransactionID,
    ) -> Result<Option<api::TransactionID>, DatabaseError> {
        let claim =
            TxClaim::find().filter(tx_claim::Column::SendTxId.eq(send_transaction_id.to_vec())).one(&self.db).await?;
        match claim {
            Some(claim) => Ok(Some(
                api::TransactionID::try_from(claim.claim_tx_id)
                    .map_err(|_| DatabaseError::Specific("invalid account id".to_string()))?,
            )),
            None => Ok(None),
        }
    }
}
