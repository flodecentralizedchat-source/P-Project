# P-Project Build Script for Windows

Write-Host "🚀 Building P-Project..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Please install Rust from https://www.rust-lang.org/" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Rust is installed" -ForegroundColor Green

# Check if wasm-pack is installed
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "📦 Installing wasm-pack..." -ForegroundColor Yellow
    cargo install wasm-pack
}

Write-Host "🏗️  Building Rust workspace..." -ForegroundColor Cyan
cargo build --release

Write-Host "🕸️  Building WebAssembly components..." -ForegroundColor Cyan
wasm-pack build p-project-web --target web --release

Write-Host "🧪 Running tests..." -ForegroundColor Cyan
cargo test

Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "To run the API server: cargo run -p p-project-api --release" -ForegroundColor White
Write-Host "To run with Docker: docker-compose up" -ForegroundColor White
Write-Host "To run tests: cargo test" -ForegroundColor White