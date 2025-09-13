FROM rust:1.89-slim AS build
WORKDIR /app

# OpenSSL ve pkg-config ekle
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock Rocket.toml ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

# runtime için sadece openssl runtime lazım
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/rust-binance-api /app/rust-binance-api

EXPOSE 8000
CMD ["./rust-binance-api"]
