use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use entity::sea_orm::prelude::DateTimeUtc;
use entity::sea_orm::{
    self, sea_query::TableCreateStatement, ConnectOptions, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
};
use entity::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set};
use entity::unix_to_datetime;
use hyper::server::accept::Accept;
use pog_proto::api;

use entity::account::{self, Entity as Account};
use entity::block::{self, Entity as Block};
use entity::pending_block::Entity as PendingBlock;
use entity::transaction::Entity as Transaction;
use entity::tx_claim::Entity as TxClaim;
use prost::Message;

#[derive(Debug)]
pub struct Sql {
    db: DatabaseConnection,
}

impl Sql {
    pub async fn connect_mock() -> Result<Sql> {
        let opt = ConnectOptions::new("sqlite::memory:".to_string());

        let db = sea_orm::Database::connect(opt).await?;
        let sql = Sql {
            db,
        };

        sql.setup_schema(sea_orm::DatabaseBackend::Sqlite).await?;

        Ok(sql)
    }

    pub async fn connect_sqlite(_cfg: &DatabaseConfig) -> Result<Sql> {
        unimplemented!("");
        // let opt = ConnectOptions::new("".to_string());

        // let db = sea_orm::Database::connect(opt).await?;

        // Ok(Sql {
        //     db,
        // })
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
        let block = Block::find_by_id(block_id.into())
            .one(&self.db)
            .await
            .map_err(DatabaseError::SeaORM)?
            .ok_or(DatabaseError::BlockNotFound)?;

        api::SignedBlock::decode(&*block.data).map_err(DatabaseError::DecodeError)
    }

    async fn get_transaction_by_id(
        &self,
        transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError> {
        let transaction = Transaction::find_by_id(transaction_id.into())
            .one(&self.db)
            .await
            .map_err(DatabaseError::SeaORM)?
            .ok_or(DatabaseError::BlockNotFound)?;

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
            .await
            .map_err(DatabaseError::SeaORM)?
            .ok_or(DatabaseError::BlockNotFound)?;

        let block = block.ok_or(DatabaseError::BlockNotFound)?;
        api::SignedBlock::decode(&*block.data).map_err(DatabaseError::DecodeError)
    }

    async fn add_block(&mut self, block: api::SignedBlock) -> Result<(), DatabaseError> {
        let block_data = block.data.clone().ok_or_else(|| DatabaseError::Specific("invalid block".to_string()))?;
        let block_id = block.get_id().map_err(|e| DatabaseError::Specific(e.to_string()))?;
        let account_id = encoding::account::generate_account_address(block.public_key.clone())
            .map_err(|_| DatabaseError::Specific("account ID could not be generated".to_string()))?;

        let _new_block = block::ActiveModel {
            height: Set(block_data.height),
            account_id_v1: Set(account_id.into()),
            balance: Set(block_data.balance),
            block_id: Set(block_id.into()),
            data: Set(block.encode_to_vec()),
            public_key: Set(block.public_key.clone()),
            signature: Set(block.signature),
            timestamp: Set(unix_to_datetime(block.timestamp)),
            version: Set(block::BlockVersion::V1),
        };

        let _new_acc = account::ActiveModel {
            public_key: Set(block.public_key),
            account_id_v1: Set(account_id.into()),
            latest_block: Set(block_id.into()),
        };

        unimplemented!("method unsupported by database backend (transaction handling is still missing)")
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
            .await
            .map_err(DatabaseError::SeaORM)?;

        match block {
            Some(block) => Ok(Some(api::SignedBlock::decode(&*block.data).map_err(DatabaseError::DecodeError)?)),
            None => Ok(None),
        }
    }

    async fn get_account_delegate(
        &self,
        _account_id: api::AccountID,
    ) -> Result<Option<api::AccountID>, DatabaseError> {
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
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_send_recipient(
        &self,
        _send_transaction_id: api::TransactionID,
    ) -> Result<Option<api::TransactionID>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }
}
