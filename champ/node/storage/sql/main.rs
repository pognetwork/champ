use crate::storage::{Database, DatabaseConfig, DatabaseError};
use anyhow::Result;
use async_trait::async_trait;
use entity::sea_orm::{
    self, sea_query::TableCreateStatement, ConnectOptions, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
};
use pog_proto::api;

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
        let opt = ConnectOptions::new("".to_string());

        let db = sea_orm::Database::connect(opt).await?;

        Ok(Sql {
            db,
        })
    }

    // not required after we've setup migrations
    pub async fn setup_schema(&self, backend: DbBackend) -> Result<()> {
        let schema = Schema::new(backend);

        let block: TableCreateStatement = schema.create_table_from_entity(entity::block::Entity);
        let pending_block: TableCreateStatement = schema.create_table_from_entity(entity::pending_block::Entity);
        let transaction: TableCreateStatement = schema.create_table_from_entity(entity::transaction::Entity);
        let tx_claim: TableCreateStatement = schema.create_table_from_entity(entity::tx_claim::Entity);

        self.db.execute(self.db.get_database_backend().build(&block)).await?;
        self.db.execute(self.db.get_database_backend().build(&pending_block)).await?;
        self.db.execute(self.db.get_database_backend().build(&transaction)).await?;
        self.db.execute(self.db.get_database_backend().build(&tx_claim)).await?;

        Ok(())
    }
}

#[async_trait]
impl Database for Sql {
    async fn get_block_by_id(&self, _block_id: api::BlockID) -> Result<api::SignedBlock, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_transaction_by_id(
        &self,
        _transaction_id: api::TransactionID,
    ) -> Result<api::Transaction, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_latest_block_by_account(
        &self,
        _account_id: api::AccountID,
    ) -> Result<api::SignedBlock, DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn add_block(&mut self, _block: api::SignedBlock) -> Result<(), DatabaseError> {
        unimplemented!("method unsupported by database backend")
    }

    async fn get_block_by_height(
        &self,
        _account_id: api::AccountID,
        _block_height: &u64,
    ) -> Result<Option<api::SignedBlock>, DatabaseError> {
        unimplemented!("method unsupported by database backend")
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
