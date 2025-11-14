pub mod ai_service;
pub mod amount;
pub mod amount_migration;
pub mod database;
pub mod iot_service;
pub mod ipfs;
pub mod merchant_service;
pub mod models;
pub mod utils;
pub mod web2_service;

#[cfg(test)]
mod ipfs_test;
#[cfg(test)]
mod merchant_service_test;

#[cfg(not(target_arch = "wasm32"))]
pub use database::*;
pub use ipfs::*;
pub use models::*;
pub use utils::*;
pub use ai_service::*;
pub use iot_service::*;
pub use merchant_service::*;
pub use web2_service::*;