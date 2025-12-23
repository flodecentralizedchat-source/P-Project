# syntax=docker/dockerfile:1.4

# Build and test p-project-core
FROM rust:1.80 AS builder

WORKDIR /app

# Faster registry downloads
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Copy the full workspace (keeps workspace relationships intact)
COPY . .

# Build the core binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build -p p-project-core --release

# Run tests in a dedicated stage to fail the build if they break
FROM builder AS tester
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo test -p p-project-core --release

# Slim runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create an unprivileged user
RUN useradd -m -u 10001 appuser

# Copy the compiled binary
COPY --from=builder /app/target/release/p-project-core-test /usr/local/bin/p-project-core

USER appuser

CMD ["p-project-core"]
