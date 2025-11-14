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

# Optimize WASM and precompress built assets (gzip + brotli)
RUN apt-get update && apt-get install -y --no-install-recommends \
      brotli \
      binaryen \
    && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    cd /app/p-project-web/pkg; \
    if [ -f p_project_web_bg.wasm ]; then \
      wasm-opt -Oz -o p_project_web_bg.opt.wasm p_project_web_bg.wasm && mv -f p_project_web_bg.opt.wasm p_project_web_bg.wasm; \
    fi; \
    find . -type f -regextype posix-extended -regex '.*\.(js|mjs|css|html|svg|json|wasm|xml|txt|webmanifest)$' \
      -exec gzip -9 -k {} \; \
      -exec brotli -Z -k {} \;

# ---- Runtime: NGINX with brotli module ----
# This base includes the ngx_brotli dynamic module
FROM fholzer/nginx-brotli:alpine AS web

# NGINX configuration (expects brotli_static/gzip_static on)
COPY nginx/nginx.conf /etc/nginx/conf.d/default.conf

# Static site + precompressed files
COPY --from=builder /app/p-project-web/pkg/ /usr/share/nginx/html/

EXPOSE 80

# Default command provided by image
