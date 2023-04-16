FROM rust:1.68-alpine as builder
WORKDIR /app
RUN apk add --no-cache musl-dev

COPY . .
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo build --release

FROM alpine:latest

ENV SIMPLE_GH_ADDR=0.0.0.0:80

# RUN mkdir /cache && apk add --no-cache ca-certificates
RUN mkdir /cache && apk add --no-cache curl

VOLUME /cache
EXPOSE 80

WORKDIR /
COPY --from=builder /app/simple-gh .

COPY docker/healthcheck.sh /healthcheck.sh
COPY docker/start.sh /start.sh

HEALTHCHECK --interval=10s --timeout=5s CMD ["/healthcheck.sh"]

CMD [ "/start.sh" ]
