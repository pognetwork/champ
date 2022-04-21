ARG ALPINE=3.15

FROM alpine:${ALPINE} AS builder
RUN apk update && apk upgrade
RUN apk add --no-cache build-base run-parts protoc ca-certificates wget \
    && update-ca-certificates

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.60.0

# https://github.com/rust-lang/docker-rust/blob/59ffcf40eed2aca0160790f0ae2f0288868e0291/1.60.0/alpine3.15/Dockerfile
RUN set -eux; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
    x86_64) rustArch='x86_64-unknown-linux-musl'; rustupSha256='bdf022eb7cba403d0285bb62cbc47211f610caec24589a72af70e1e900663be9' ;; \
    aarch64) rustArch='aarch64-unknown-linux-musl'; rustupSha256='89ce657fe41e83186f5a6cdca4e0fd40edab4fd41b0f9161ac6241d49fbdbbbe' ;; \
    *) echo >&2 "unsupported architecture: $apkArch"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

WORKDIR /usr/src/champ
COPY . .

RUN rustc --version
RUN cargo build --release --bin champ-node

FROM alpine:${ALPINE}
RUN apk update && apk upgrade

COPY --from=builder \
    /usr/src/champ/target/release/champ-node \
    /usr/local/bin/

EXPOSE 50048
EXPOSE 50049
EXPOSE 50050
EXPOSE 50051

ENTRYPOINT ["/usr/local/bin/champ-node"]
