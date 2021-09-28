set positional-arguments

default:
  @just --list

node *FLAGS:
  cargo run --bin champ-node -- {{FLAGS}}

wallet *FLAGS:
  cargo run --bin champ-wallet -- {{FLAGS}}

scylla:
  docker-compose -f ./scripts/scylla.docker-compose.yml up -d

next-version:
  echo $([ $(convco version) == $(convco version --bump) ] && convco version --patch || convco version --bump)

generate-changelog version:
  convco changelog v{{version}} | diff --changed-group-format='%<' --unchanged-group-format='' - CHANGELOG.md > RELEASE_NOTES.md || true
  convco changelog > CHANGELOG.md

generate-next-changelog:
  just generate-changelog $(just next-version)

release:
  cargo release $(just next-version)