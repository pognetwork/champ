# Storage Layer

- Pluggable database storage interface
- Most logic should be in application code
- Starting with ScyllaDB (whic is essentially a faster Cassandra)
- Later adding rocksdb backend for improved performance and portability for single node
