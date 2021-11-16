use pog_proto::api::{signed_block::BlockData, SignedBlock};

use common::storage::TestStorage;
mod common;

#[tokio::test]
async fn test_mock() {
    let _ = TestStorage::new_mock().await;
}

#[tokio::test]
async fn add_and_find_blocks() {
    let mut db = TestStorage::new().await.db;

    let block = SignedBlock {
        data: Some(BlockData {
            ..Default::default()
        }),
        ..Default::default()
    };

    let block_id = block.get_id().expect("should generate block id");
    db.add_block(block.clone()).await.expect("should add block to database");
    let block_res = db.get_block_by_id(block_id).await.expect("should return block");
    assert_eq!(block_res, block);
}
