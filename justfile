default:
  @just --list
mkdocs:
	docker run --rm -it -p 8000:8000 -v ${PWD}:/docs squidfunk/mkdocs-material
node *FLAGS:
  cargo run --bin champ-node -- {{FLAGS}}
wallet *FLAGS:
  cargo run --bin champ-wallet -- {{FLAGS}}