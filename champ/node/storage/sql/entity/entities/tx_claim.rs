use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tx_claim")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub send_tx_id: Vec<u8>,
    #[sea_orm(primary_key)]
    pub claim_tx_id: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::SendTxId",
        to = "super::transaction::Column::TransactionId"
    )]
    SendTransaction,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::ClaimTxId",
        to = "super::transaction::Column::TransactionId"
    )]
    ClaimTransaction,
}

impl ActiveModelBehavior for ActiveModel {}
