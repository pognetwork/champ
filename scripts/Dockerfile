ARG TOOLCHAIN=1.60
ARG ALPINE=3.15

FROM rust:${TOOLCHAIN}-alpine${ALPINE} AS builder
RUN apk update && apk upgrade
RUN apk add --no-cache build-base protoc

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
