# Build WASM package and serve via nginx
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

# Build the WebAssembly package
RUN wasm-pack build p-project-web --target web --out-dir pkg

# Precompress built assets (gzip + brotli)
RUN apt-get update && apt-get install -y --no-install-recommends \
      brotli \
    && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    cd /app/p-project-web/pkg; \
    find . -type f -regextype posix-extended -regex '.*\.(js|mjs|css|html|svg|json|wasm|xml|txt|webmanifest)$' \
      -exec gzip -9 -k {} \; \
      -exec brotli -Z -k {} \;

# Static web stage: Nginx serving built WASM package
FROM fholzer/nginx-brotli:alpine

# Copy built web assets from builder
COPY --from=builder /app/p-project-web/pkg/ /usr/share/nginx/html/

# Provide nginx config with proper MIME types for WASM
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf
