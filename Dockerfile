FROM rust:1.64.0-alpine3.16 as build

ENV LANG=C.UTF-8 \
    TZ=UTC \
    TERM=xterm-256color \
    CARGO_HOME="/root/.cargo" \
    USER="root"

RUN mkdir -pv "${CARGO_HOME}" \
    && rustup set profile minimal

RUN apk add --no-cache musl-dev

RUN USER=root cargo new --bin /app
WORKDIR /app

COPY ./Cargo.* ./

RUN cargo build --release \
    && find . -not -path "./target*" -delete

COPY . .
RUN touch src/main.rs

RUN cargo build --release

FROM alpine:3.16

ENV ROCKET_PROFILE="release" \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=80

# RUN mkdir /cache && apk add --no-cache ca-certificates
RUN mkdir /cache && apk add --no-cache curl

VOLUME /cache
EXPOSE 80

WORKDIR /
COPY --from=build /app/target/release/simple-gh .

COPY docker/healthcheck.sh /healthcheck.sh
COPY docker/start.sh /start.sh

HEALTHCHECK --interval=60s --timeout=10s CMD ["/healthcheck.sh"]

CMD [ "/start.sh" ]