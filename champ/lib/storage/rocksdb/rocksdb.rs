use std::sync::Arc;

use crate::Database;

pub struct RocksDB {
    db: Arc<rocksdb::DB>,
}

impl RocksDB {}
impl Database for RocksDB {}
