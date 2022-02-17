use champ_node::storage::{self, Database, DatabaseConfig};
use crypto::signatures::ed25519::{create_public_key, create_signature, generate_private_key};
use pog_proto::api::{signed_block::BlockData, transaction::TxClaim, SignedBlock, Transaction};

pub struct TestStorage {
    pub db: Box<dyn Database>,
}

pub struct TestAccount {
    private_key: [u8; 32],
    public_key: [u8; 32],
}

const GENESIS_ID: &[u8; 32] = b"00000000000000000000000000000000";

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

    pub fn mock_sign_blockdata(
        block_data: BlockData,
        index: u64,
        public_key: &[u8],
        private_key: &[u8],
    ) -> SignedBlock {
        let signature =
            create_signature(&block_data.unique_bytes().unwrap(), private_key).expect("should create signature");
        let timestamp = 1637000000 + index * 2000;
        SignedBlock {
            data: Some(block_data),
            public_key: public_key.to_vec(),
            signature: signature.to_vec(),
            timestamp,
        }
    }

    pub fn mock_accounts(count: u8) -> Vec<TestAccount> {
        let mut accounts: Vec<TestAccount> = vec![];

        for _ in 0..count {
            let private_key = generate_private_key().expect("should generate private key");
            let public_key = create_public_key(&private_key).expect("should calculate public key");
            accounts.push(TestAccount {
                private_key,
                public_key,
            })
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
            let block = TestStorage::mock_sign_blockdata(
                genesis_block_data,
                n.into(),
                &account.public_key,
                &account.private_key,
            );
            self.db.add_block(block).await.expect("should add block");
        }
    }
}
