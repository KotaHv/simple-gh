FROM --platform=$BUILDPLATFORM rust:1.64.0-alpine3.16 as build

ARG TARGETARCH
RUN case "$TARGETARCH" in \
    "amd64") \
    MUSL="x86_64-linux-musl"; \
    echo x86_64-unknown-linux-musl > /rust_target.txt \
    ;; \
    "arm64") \
    MUSL="aarch64-linux-musl"; \
    echo aarch64-unknown-linux-musl > /rust_target.txt \
    ;; \
    "arm") \
    MUSL="arm-linux-musleabihf"; \
    echo armv7-unknown-linux-musleabihf > /rust_target.txt \
    ;; \
    *) \
    echo "Doesn't support $TARGETARCH architecture" \
    exit 1 ;; \
    esac \
    && echo "$MUSL" \
    && wget -qO- "https://musl.cc/$MUSL-cross.tgz" | tar -xzC /root/ \
    && PATH="/root/$MUSL-cross/bin:$PATH" \
    && CC=/root/$MUSL-cross/bin/$MUSL-gcc \
    && echo "$CC" > /cc.txt

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

RUN rustup target add $(cat /rust_target.txt)

RUN RUSTFLAGS="-C linker=$(cat /cc.txt)" CC=$(cat /cc.txt) cargo build --release --target $(cat /rust_target.txt) \
    && find . -not -path "./target*" -delete

COPY . .
RUN touch src/main.rs

RUN RUSTFLAGS="-C linker=$(cat /cc.txt)" CC=$(cat /cc.txt) cargo build --release --target $(cat /rust_target.txt)
RUN mv target/$(cat /rust_target.txt)/release/simple-gh .

FROM alpine:3.16

ENV ROCKET_PROFILE="release" \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=80

# RUN mkdir /cache && apk add --no-cache ca-certificates
RUN mkdir /cache && apk add --no-cache curl

VOLUME /cache
EXPOSE 80

WORKDIR /
COPY --from=build /app/simple-gh .

COPY docker/healthcheck.sh /healthcheck.sh
COPY docker/start.sh /start.sh

HEALTHCHECK --interval=60s --timeout=10s CMD ["/healthcheck.sh"]

CMD [ "/start.sh" ]