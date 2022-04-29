set positional-arguments

default:
  @just --list

node *FLAGS:
  cargo run --bin champ-node -- {{FLAGS}}

wallet *FLAGS:
  cargo run --bin champ-wallet -- {{FLAGS}}

next-version:
  echo $([ $(convco version) == $(convco version --bump) ] && convco version --patch || convco version --bump)

generate-changelog version:
  git cliff --tag {{version}} --output CHANGELOG.md

generate-release-notes version:
  git cliff --unreleased --tag {{version}} --output RELEASE_NOTES.md

release:
  cargo release $(just next-version)

build-docker platform dockerplatform name tag:
  docker buildx build --platform {{dockerplatform}} -f ./scripts/Dockerfile -t {{name}}:{{tag}} . --build-arg PLATFORM={{platform}}

docker-make-image platform dockerplatform name tag:
  docker buildx build --platform {{dockerplatform}} -f ./scripts/Dockerfile -t {{name}}:{{tag}} . --build-arg PLATFORM={{platform}} --push

image_name := "ghcr.io/pognetwork/champ"
docker tag hash:
  @echo "Building for arm64"
  just docker-make-image aarch64-musl linux/arm64 {{image_name}} {{tag}}-arm64

  @echo "Building for X86"
  just docker-make-image x86_64-musl linux/amd64 {{image_name}} {{tag}}-amd64

  docker manifest create {{image_name}}:{{tag}} {{image_name}}:{{tag}}-amd64 {{image_name}}:{{tag}}-arm64
  docker manifest create {{image_name}}:{{tag}}-{{hash}} {{image_name}}:{{tag}}-amd64 {{image_name}}:{{tag}}-arm64
  docker manifest push {{image_name}}:{{tag}}
  docker manifest push {{image_name}}:{{tag}}-{{hash}}