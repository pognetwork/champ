FROM alpine:3.14
RUN apk update && apk upgrade
ADD ./champ-node .

EXPOSE 50048
EXPOSE 50049
EXPOSE 50050
EXPOSE 50051

ENTRYPOINT ["./champ-node"]
