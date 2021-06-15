default:
  @just --list
node *FLAGS:
  cargo run --bin champ-node -- {{FLAGS}}
wallet *FLAGS:
  cargo run --bin champ-wallet -- {{FLAGS}}