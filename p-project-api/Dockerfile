# Build p-project-api binary
FROM rust:1.70 AS builder

WORKDIR /app

# Leverage Docker cache by copying manifests first
COPY Cargo.toml Cargo.lock ./
COPY p-project-core/Cargo.toml p-project-core/Cargo.toml
COPY p-project-contracts/Cargo.toml p-project-contracts/Cargo.toml
COPY p-project-api/Cargo.toml p-project-api/Cargo.toml
COPY p-project-dao/Cargo.toml p-project-dao/Cargo.toml
COPY p-project-staking/Cargo.toml p-project-staking/Cargo.toml
COPY p-project-airdrop/Cargo.toml p-project-airdrop/Cargo.toml
COPY p-project-bridge/Cargo.toml p-project-bridge/Cargo.toml
COPY p-project-web/Cargo.toml p-project-web/Cargo.toml

# Pre-fetch dependencies
RUN cargo fetch

# Copy full source
COPY . .

# Build the API in release mode
RUN cargo build -p p-project-api --release

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create an unprivileged user
RUN useradd -m -u 10001 appuser

# Copy API binary
COPY --from=builder /app/target/release/p-project-api ./p-project-api

USER appuser

EXPOSE 3000
CMD ["./p-project-api"]

