# P-Project Build Script for Windows

Write-Host "Building P-Project..."

# Build the entire workspace
Write-Host "Building Rust workspace..."
cargo build

# Check if wasm-pack is installed
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "wasm-pack could not be found, installing..."
    cargo install wasm-pack
}

# Build WASM components
Write-Host "Building WASM components..."
wasm-pack build p-project-web --target web

Write-Host "Build complete!"
Write-Host ""
Write-Host "To run the API server: cargo run -p p-project-api"
Write-Host "To run tests: cargo test"