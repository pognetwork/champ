use pog_proto::api;
use pog_proto::DecodeError;
use pog_proto::Message;

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

    pub height: u64,
    pub account_id_v1: Vec<u8>,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub version: BlockVersion,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: u64,
    pub data: Vec<u8>,
}

impl TryInto<api::SignedBlock> for Model {
    type Error = DecodeError;

    fn try_into(self) -> Result<pog_proto::api::SignedBlock, Self::Error> {
        let data = api::BlockData::decode(&*self.data)?;
        let header = api::BlockHeader {
            public_key: self.public_key,
            signature: self.signature,
            timestamp: self.timestamp.timestamp() as u64,
        };

        Ok(api::SignedBlock {
            data_raw: self.data,
            data,
            header,
        })
    }
}

impl<'a> TryInto<api::SignedBlock> for &Model {
    type Error = DecodeError;

    fn try_into(self) -> Result<pog_proto::api::SignedBlock, Self::Error> {
        let data = api::BlockData::decode(&*self.data)?;
        let header = api::BlockHeader {
            public_key: self.public_key.clone(),
            signature: self.signature.clone(),
            timestamp: self.timestamp.timestamp() as u64,
        };

        Ok(api::SignedBlock {
            data_raw: self.data.clone(),
            data,
            header,
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::PublicKey",
        to = "super::account::Column::PublicKey"
    )]
    Account,

    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
