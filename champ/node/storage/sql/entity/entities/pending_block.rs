use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pending_blocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub block_id: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: u64,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
