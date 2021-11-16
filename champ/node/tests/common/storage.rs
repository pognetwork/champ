use champ_node::storage::{self, Database, DatabaseConfig};
use crypto::signatures::ed25519::{create_public_key, create_signature, generate_private_key};
use pog_proto::api::{signed_block::BlockData, transaction::TxClaim, SignedBlock, Transaction};

pub struct TestStorage {
    pub db: Box<dyn Database>,
}

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

    pub async fn mock(&mut self) {
        let mut accounts: Vec<([u8; 32], [u8; 32])> = vec![];

        for _ in 0..10 {
            let private_key = generate_private_key().expect("should generate private key");
            let public_key = create_public_key(&private_key).expect("should calculate public key");

            accounts.push((private_key, public_key))
        }

        for n in 0..10 {
            let genesis_block = BlockData {
                balance: n + 1 * 100,
                height: 0,
                previous: None,
                signature_type: pog_proto::api::SigType::Ed25519.into(),
                version: pog_proto::api::BlockVersion::V1.into(),
                transactions: vec![Transaction {
                    data: Some(pog_proto::api::transaction::Data::TxCollect(TxClaim {
                        transaction_id: b"000000000000000000000000000000000000000".to_vec(),
                    })),
                }],
            };

            let acc = accounts.get(n as usize).unwrap();
            let signature =
                create_signature(&genesis_block.unique_bytes().unwrap(), &acc.0).expect("should create signature");

            let timestamp = 1637000000 + n * 2000;
            self.db
                .add_block(SignedBlock {
                    data: Some(genesis_block),
                    public_key: acc.1.to_vec(),
                    signature: signature.to_vec(),
                    timestamp,
                })
                .await
                .expect("should add block");
        }
    }
}
