FROM alpine:3.14
RUN apk update && apk upgrade
ADD ./target/x86_64-unknown-linux-musl/release/champ-node .

EXPOSE 50049
EXPOSE 50050
EXPOSE 50051

CMD [ "./champ-node" ]
