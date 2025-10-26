# Use Rust official image as base
FROM rust:1.70 as builder

# Create app directory
WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal base image for the runtime
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/p-project-api ./p-project-api

# Copy WebAssembly components
COPY --from=builder /app/p-project-web/pkg ./pkg

# Expose port
EXPOSE 3000

# Run the application
CMD ["./p-project-api"]