#[cfg(target_arch = "wasm32")]
mod wasm_components;

#[cfg(target_arch = "wasm32")]
pub use wasm_components::*;

#[cfg(not(target_arch = "wasm32"))]
mod server_components;

#[cfg(not(target_arch = "wasm32"))]
pub use server_components::*;
