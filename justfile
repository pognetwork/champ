default:
  @just --list
node *FLAGS:
  cargo run --bin champ-node -- {{FLAGS}}
wallet *FLAGS:
  cargo run --bin champ-wallet -- {{FLAGS}}
scylla:
  docker-compose -f ./scripts/scylla.docker-compose.yml up -d