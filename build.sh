#!/bin/bash

# P-Project Build Script

echo "Building P-Project..."

# Build the entire workspace
echo "Building Rust workspace..."
cargo build

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found, installing..."
    cargo install wasm-pack
fi

# Build WASM components
echo "Building WASM components..."
wasm-pack build p-project-web --target web

echo "Build complete!"
echo ""
echo "To run the API server: cargo run -p p-project-api"
echo "To run tests: cargo test"