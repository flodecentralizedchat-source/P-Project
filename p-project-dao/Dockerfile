# Build and test p-project-dao
FROM rust:1.80-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests up front to leverage Docker layer caching
COPY Cargo.toml ./
COPY p-project-core/Cargo.toml p-project-core/Cargo.toml
COPY p-project-contracts/Cargo.toml p-project-contracts/Cargo.toml
COPY p-project-api/Cargo.toml p-project-api/Cargo.toml
COPY p-project-dao/Cargo.toml p-project-dao/Cargo.toml
COPY p-project-staking/Cargo.toml p-project-staking/Cargo.toml
COPY p-project-airdrop/Cargo.toml p-project-airdrop/Cargo.toml
COPY p-project-bridge/Cargo.toml p-project-bridge/Cargo.toml
COPY p-project-web/Cargo.toml p-project-web/Cargo.toml

# Copy full workspace
COPY . .

# Run the DAO crate tests by default
CMD ["cargo", "test", "-p", "p-project-dao"]
