# Builder stage: compile Rust backend and WASM frontend
FROM rust:1.70 AS builder

WORKDIR /app

# Install tools needed to build WASM
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack --locked

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

# Build the WebAssembly package
RUN wasm-pack build p-project-web --target web --out-dir pkg

# Runtime stage: minimal image running as non-root
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create an unprivileged user
RUN useradd -m -u 10001 appuser

# Copy binaries and assets from builder
COPY --from=builder /app/target/release/p-project-api ./p-project-api
COPY --from=builder /app/p-project-web/pkg ./pkg

# Fix ownership and drop privileges
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000
CMD ["./p-project-api"]

# Static web stage: Nginx serving built WASM package
FROM nginx:alpine AS web-static

# Copy built web assets from builder
COPY --from=builder /app/p-project-web/pkg /usr/share/nginx/html

# Provide nginx config with proper MIME types for WASM
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf
