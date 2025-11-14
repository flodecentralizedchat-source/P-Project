#[cfg(not(target_arch = "wasm32"))]
pub mod database;
pub mod ipfs;
pub mod models;
pub mod utils;
pub mod ai_service;
pub mod iot_service;
pub mod web2_service;

#[cfg(test)]
mod ipfs_test;

#[cfg(not(target_arch = "wasm32"))]
pub use database::*;
pub use ipfs::*;
pub use models::*;
pub use utils::*;
pub use ai_service::*;
pub use iot_service::*;
pub use web2_service::*;