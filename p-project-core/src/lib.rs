#[cfg(not(target_arch = "wasm32"))]
pub mod database;
pub mod models;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub use database::*;
pub use models::*;
pub use utils::*;
