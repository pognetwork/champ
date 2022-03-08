use champ_node::storage::{self, Database, DatabaseConfig};
use crypto::signatures::ed25519::{create_public_key, create_signature, generate_private_key};
use pog_proto::{
    api::{transaction::TxClaim, BlockData, BlockHeader, SignedBlock, Transaction},
    Message,
};

pub struct TestStorage {
    pub db: Box<dyn Database>,
}

pub struct TestAccount {
    private_key: [u8; 32],
    public_key: [u8; 32],
}

pub const GENESIS_ID: &[u8; 32] = b"00000000000000000000000000000000";

impl TestStorage {
    pub async fn new() -> Self {
        let db = storage::new(&DatabaseConfig {
            kind: storage::Databases::Sled,
            temporary: Some(true),
            ..Default::default()
        })
        .await
        .expect("should create database");

        Self {
            db,
        }
    }

    pub async fn new_mock() -> Self {
        let mut test_storage = Self::new().await;
        test_storage.mock().await;
        test_storage
    }

    pub fn mock_sign_data(block_data: &[u8], index: u64, public_key: &[u8], private_key: &[u8]) -> SignedBlock {
        let signature = create_signature(block_data, private_key).expect("should create signature");
        let timestamp = 1637000000 + index * 2000;
        SignedBlock::new(
            BlockHeader {
                public_key: public_key.to_vec(),
                signature: signature.to_vec(),
                timestamp,
            },
            BlockData::decode(block_data).unwrap(),
        )
    }

    pub fn mock_account() -> TestAccount {
        let private_key = generate_private_key().expect("should generate private key");
        let public_key = create_public_key(&private_key).expect("should calculate public key");
        TestAccount {
            private_key,
            public_key,
        }
    }

    pub fn mock_accounts(count: u8) -> Vec<TestAccount> {
        let mut accounts: Vec<TestAccount> = vec![];

        for _ in 0..count {
            accounts.push(TestStorage::mock_account())
        }
        accounts
    }

    pub fn mock_blockdata(balance: u64, height: u64, previous: &[u8], transactions: Vec<Transaction>) -> BlockData {
        BlockData {
            balance,
            height,
            previous: previous.to_vec(),
            signature_type: pog_proto::api::SigType::Ed25519.into(),
            version: pog_proto::api::BlockVersion::V1.into(),
            transactions,
        }
    }

    pub fn mock_simple_signed_block() -> SignedBlock {
        let account = TestStorage::mock_account();
        let block_data = TestStorage::mock_blockdata(
            100u64,
            0,
            GENESIS_ID,
            vec![Transaction {
                data: Some(pog_proto::api::transaction::Data::TxClaim(TxClaim {
                    send_transaction_id: GENESIS_ID.to_vec(),
                })),
            }],
        );

        TestStorage::mock_sign_data(&block_data.encode_to_vec(), 0, &account.public_key, &account.private_key)
    }

    pub async fn mock(&mut self) {
        let count = 10;
        let accounts = TestStorage::mock_accounts(count);

        let genesis_tx = pog_proto::api::transaction::Data::TxClaim(TxClaim {
            send_transaction_id: GENESIS_ID.to_vec(),
        });

        let genesis_txs = vec![Transaction {
            data: Some(genesis_tx),
        }];

        for n in 0..count {
            let genesis_block_data = TestStorage::mock_blockdata(100 + n as u64, 0, GENESIS_ID, genesis_txs.clone());

            let account = accounts.get(n as usize).unwrap();
            let block = TestStorage::mock_sign_data(
                &genesis_block_data.encode_to_vec(),
                n.into(),
                &account.public_key,
                &account.private_key,
            );
            self.db.add_block(block).await.expect("should add block");
        }
    }
}
