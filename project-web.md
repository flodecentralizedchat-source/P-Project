## Project Web Features

- **WASM-focused crate**: `p-project-web` is a library crate (`cdylib` + `rlib`) that targets WebAssembly while reusing workspace dependencies (`serde`, `tokio`, `wasm-bindgen`, `web-sys`, `js-sys`) and shared Rust modules (`p-project-core`, `p-project-contracts`).
- **Dual-entry module**: `src/lib.rs` conditionally exposes either browser/WASM bindings (`wasm_components`) or a lightweight server stub (`server_components`) depending on the compilation target.
- **Web user wrapper**: `wasm_components::WebUser` wraps the core `User` model for JS, including constructors/getters and a `short_wallet_address` helper that formats addresses via `shorten_wallet_address`.
- **JavaScript bindings**: the wasm module exposes helper functions such as `greet(name)` and `initialize_app()` along with console logging, all annotated with `wasm-bindgen` so they can be invoked from JavaScript.
- **Server fallback**: the `server_components` module provides a `server_init()` placeholder to keep the crate usable from non-wasm targets without dragging in wasm-specific dependencies.
