ARG IMG_BASE=messense/rust-musl-cross
ARG TOOLCHAIN=1.60.0
ARG IMG_ARCH=x86_64-musl

FROM ${IMG_BASE}:${IMG_ARCH} AS builder

WORKDIR /home/rust/src
ADD . ./

RUN rustc --version
RUN cargo build --release --bin champ-node
RUN mv target/$CARGO_BUILD_TARGET out
RUN ls out

FROM alpine:3.14
RUN apk update && apk upgrade
COPY --from=builder \
    /home/rust/src/out/release/champ-node \
    /usr/local/bin/

EXPOSE 50048
EXPOSE 50049
EXPOSE 50050
EXPOSE 50051

ENTRYPOINT ["/usr/local/bin/champ-node"]
