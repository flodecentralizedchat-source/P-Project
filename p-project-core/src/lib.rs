pub mod ai_service;
pub mod amount;
pub mod amount_migration;
pub mod creator_rewards_service;
pub mod credit_service;
pub mod csr_service;
pub mod dao_governance_service;
pub mod database;
pub mod digital_governance_service;
pub mod game_currency_service;
pub mod government_service;
pub mod identity_service;
pub mod iot_service;
pub mod ipfs;
pub mod merchant_service;
pub mod models;
pub mod nft_collectibles_service;
pub mod p2e_service;
pub mod payroll_service;
pub mod school_programs_service;
pub mod sponsorship_service;
pub mod tokenized_assets_service;
pub mod tokenomics_service;
pub mod utils;
pub mod web2_service;

#[cfg(test)]
mod creator_rewards_service_test;
#[cfg(test)]
mod credit_service_test;
#[cfg(test)]
mod csr_service_test;
#[cfg(test)]
mod dao_governance_service_test;
#[cfg(test)]
mod digital_governance_service_test;
#[cfg(test)]
mod game_currency_service_test;
#[cfg(test)]
mod government_service_test;
#[cfg(test)]
mod identity_service_test;
#[cfg(test)]
mod ipfs_test;
#[cfg(test)]
mod merchant_service_test;
#[cfg(test)]
mod nft_collectibles_service_test;
#[cfg(test)]
mod p2e_service_test;
#[cfg(test)]
mod payroll_service_test;
#[cfg(test)]
mod school_programs_service_test;
#[cfg(test)]
mod sponsorship_service_test;
#[cfg(test)]
mod tokenized_assets_service_test;
#[cfg(test)]
mod tokenomics_service_test;
#[cfg(test)]
mod web2_service_test;

pub use ai_service::*;
pub use creator_rewards_service::*;
pub use credit_service::*;
pub use csr_service::*;
pub use dao_governance_service::*;
#[cfg(not(target_arch = "wasm32"))]
pub use database::*;
pub use digital_governance_service::*;
pub use game_currency_service::*;
pub use government_service::*;
pub use identity_service::*;
pub use iot_service::*;
pub use ipfs::*;
pub use merchant_service::*;
pub use models::*;
pub use nft_collectibles_service::*;
pub use p2e_service::*;
pub use payroll_service::*;
pub use school_programs_service::*;
pub use sponsorship_service::*;
pub use tokenized_assets_service::*;
pub use tokenomics_service::*;
pub use utils::*;
pub use web2_service::*;
