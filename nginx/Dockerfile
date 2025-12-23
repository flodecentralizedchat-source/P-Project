# syntax=docker/dockerfile:1

# ---- Builder: build WASM + precompress assets ----
FROM rust:1.70 AS builder

WORKDIR /app

# Install wasm toolchain
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack --locked

# Speed up build by caching deps
COPY Cargo.toml Cargo.lock ./
COPY p-project-core/Cargo.toml p-project-core/Cargo.toml
COPY p-project-contracts/Cargo.toml p-project-contracts/Cargo.toml
COPY p-project-web/Cargo.toml p-project-web/Cargo.toml
RUN cargo fetch

# Copy sources and build
COPY . .
RUN wasm-pack build p-project-web --target web --out-dir pkg --release

# Add a minimal SPA shell to the built pkg
COPY nginx/index.html p-project-web/pkg/index.html
COPY nginx/app.js p-project-web/pkg/app.js

# Precompress built assets (gzip only, since we're using standard nginx)
RUN apt-get update && apt-get install -y --no-install-recommends \
      gzip \
    && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    cd /app/p-project-web/pkg; \
    find . -type f -regextype posix-extended -regex '.*\.(js|mjs|css|html|svg|json|wasm|xml|txt|webmanifest)$' \
      -exec gzip -9 -k {} \;

# ---- Runtime: Standard NGINX ----
FROM nginx:alpine AS web

# NGINX configuration
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf

# Static site + precompressed files
COPY --from=builder /app/p-project-web/pkg/ /usr/share/nginx/html/

EXPOSE 80

# Default command provided by image