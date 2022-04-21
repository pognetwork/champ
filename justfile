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
  git cliff --tag {{version}} --output CHANGELOG.md

generate-release-notes version:
  git cliff --unreleased --tag {{version}} --output RELEASE_NOTES.md

release:
  cargo release $(just next-version)

release-docker platform image tag hash:
  docker buildx build --platform {{platform}} -t {{image}}:{{tag}} -t {{image}}:{{tag}}-{{hash}} -f ./scripts/Dockerfile . --push

build-local-docker image tag:
  docker buildx build -t {{image}}:{{tag}} -f ./scripts/Dockerfile . --load
