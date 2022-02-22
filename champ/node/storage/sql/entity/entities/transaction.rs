use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
#[allow(clippy::enum_variant_names)]
// pog_proto::api::transaction::Data
pub enum TxType {
    #[sea_orm(num_value = 0)]
    TxOpen,
    #[sea_orm(num_value = 1)]
    TxSend,
    #[sea_orm(num_value = 2)]
    TxCollect,
    #[sea_orm(num_value = 3)]
    TxDelegate,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub transaction_id: Vec<u8>,
    #[sea_orm(indexed)]
    pub block_id: Vec<u8>,
    pub tx_type: TxType,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::block::Entity", from = "Column::BlockId", to = "super::block::Column::BlockId")]
    Block,
}

impl Related<super::block::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Block.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
