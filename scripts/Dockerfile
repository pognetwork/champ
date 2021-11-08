FROM alpine:3.14
RUN apk update && apk upgrade
ADD ./target/x86_64-unknown-linux-musl/release/champ-node .
CMD [ "./champ-node" ]