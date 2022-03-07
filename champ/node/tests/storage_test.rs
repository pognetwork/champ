mod common;
use common::storage::TestStorage;

#[tokio::test]
async fn test_mock() {
    let _ = TestStorage::new_mock().await;
}

#[tokio::test]
async fn test_add_block() {
    let mut db = TestStorage::new().await.db;
    let block = TestStorage::mock_simple_signed_block();

    let block_id = block.get_id();
    db.add_block(block.clone()).await.expect("should add block to database");
    let block_res = db.get_block_by_id(block_id).await.expect("should return block");
    assert_eq!(block_res, block);
}

// #[tokio::test]
// async fn test_get_send_recipient() {
//     let mut db = TestStorage::new().await.db;
// }
