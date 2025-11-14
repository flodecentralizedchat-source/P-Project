#!/bin/bash

# P-Project Build Script

echo "🚀 Building P-Project..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null
then
    echo "❌ Rust is not installed. Please install Rust from https://www.rust-lang.org/"
    exit 1
fi

echo "✅ Rust is installed"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
fi

echo "🏗️  Building Rust workspace..."
cargo build --release

echo "🕸️  Building WebAssembly components..."
wasm-pack build p-project-web --target web --release

echo "🧪 Running tests..."
cargo test

echo "✅ Build complete!"
echo ""
echo "To run the API server: cargo run -p p-project-api --release"
echo "To run with Docker: docker-compose up"
echo "To run tests: cargo test"