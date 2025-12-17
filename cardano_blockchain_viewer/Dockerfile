# Build stage - Use Rust nightly for edition2024
FROM rustlang/rust:nightly as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage - Use Debian Sid (unstable) to match nightly's GLIBC
FROM debian:sid-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl -L https://github.com/txpipe/oura/releases/download/v1.9.1/oura-x86_64-unknown-linux-gnu.tar.gz \
    | tar xz -C /usr/local/bin

RUN useradd -m -u 1000 appuser

WORKDIR /app

COPY --from=builder /app/target/release/cardano_blockchain_viewer /app/cardano_blockchain_viewer

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["./cardano_blockchain_viewer"]