# Build and test p-project-contracts
FROM rust:1.80

WORKDIR /app

# Install tools needed for testing
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    llvm \
    libudev-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Run tests directly
CMD ["cargo", "test", "-p", "p-project-contracts"]