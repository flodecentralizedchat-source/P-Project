pub mod airdrop;
pub mod staking;
pub mod token;

pub use airdrop::{AirdropContract, MerkleTree};
pub use staking::StakingContract;
pub use token::{PProjectToken, TokenEvent};

#[cfg(not(target_arch = "wasm32"))]
pub mod airdrop_db;

#[cfg(not(target_arch = "wasm32"))]
pub mod staking_db;

#[cfg(not(target_arch = "wasm32"))]
pub mod token_db;

#[cfg(not(target_arch = "wasm32"))]
pub mod db_adapters;

#[cfg(not(target_arch = "wasm32"))]
pub use airdrop_db::AirdropDbAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub use staking_db::StakingDbAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub use token_db::TokenDbAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub use db_adapters::DatabaseManager;

#[cfg(test)]
mod token_test;

#[cfg(test)]
mod staking_test;

#[cfg(test)]
mod airdrop_test;

#[cfg(test)]
mod db_integration_test;

#[cfg(test)]
mod error_tests;

#[cfg(test)]
mod serialization_tests;

#[cfg(test)]
mod full_integration_tests;
