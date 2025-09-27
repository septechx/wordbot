# Builder stage
FROM rust:1.87-slim as builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/wordbot .

RUN apt-get update && apt-get install -y openssl

RUN useradd -m -u 1000 bot
USER bot

ENTRYPOINT ["wordbot"] 
