use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
// pog_proto::api::BlockVersion
pub enum BlockVersion {
    #[sea_orm(num_value = 0)]
    V1 = 0,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "blocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub block_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub version: BlockVersion,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: u64,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl ActiveModelBehavior for ActiveModel {}
