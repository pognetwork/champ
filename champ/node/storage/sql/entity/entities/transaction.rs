use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub block_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::block::Entity", from = "Column::BlockId", to = "super::block::Column::Id")]
    Block,
}

impl Related<super::block::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Block.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
