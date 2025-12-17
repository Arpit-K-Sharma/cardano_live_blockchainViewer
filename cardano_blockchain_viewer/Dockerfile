
##### Build stage #############################################################
# Use nightly Rust because Cargo.toml uses edition2024 (nightly-only for now)
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest and source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the backend (binary name: cardano_blockchain_viewer)
RUN cargo build --release

# Install the Oura CLI so `Command::new("oura")` works inside the container
RUN cargo install oura --version 2.0.0


##### Runtime stage ###########################################################
# Use Debian sid to match the glibc version used in the nightly builder image,
# so the Oura binary (built in the first stage) can run without GLIBC errors.
FROM debian:sid-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled backend binary
COPY --from=builder /app/target/release/cardano_blockchain_viewer /app/cardano_blockchain_viewer

# Copy Oura CLI into PATH
COPY --from=builder /usr/local/cargo/bin/oura /usr/local/bin/oura

# Optional: verify Oura is available
RUN oura --version

# Default network settings (can still be overridden at runtime)
ENV CARDANO_NETWORK=preprod
ENV CARDANO_RELAY=preprod-node.world.dev.cardano.org:30000
ENV CARDANO_MAGIC=pre-prod

# Run as non‑root for safety
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./cardano_blockchain_viewer"]
