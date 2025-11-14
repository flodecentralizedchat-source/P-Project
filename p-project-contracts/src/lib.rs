pub mod airdrop;
pub mod staking;
pub mod token;
pub mod treasury;
pub mod vesting;
pub mod price_simulation; // Add the new price simulation module
pub mod nft; // Add the new NFT module
pub mod liquidity_pool; // Add the new liquidity pool module
pub mod l2_rollup; // Add the new L2 rollup module
pub mod l2_cross_chain; // Add the new L2 cross-chain module
pub mod l2_state_management; // Add the new L2 state management module
pub mod l2_batching; // Add the new L2 batching module
pub mod formal_verification; // Add the new formal verification module
pub mod l2_model_checking; // Add the new L2 model checking module
pub mod theorem_proving; // Add the new theorem proving module
pub mod comprehensive_verification; // Add the new comprehensive verification module
pub mod advanced_verification; // Add the new advanced verification module
pub mod comprehensive_test_suite; // Add the new comprehensive test suite module
pub mod advanced_cryptography; // Add the new advanced cryptography module
pub mod load_testing; // Add the new load testing module

// Re-export the main contract types
pub use airdrop::{AirdropContract, MerkleTree};
pub use staking::StakingContract;
pub use token::PProjectToken;
pub use treasury::{LiquidityMiningProgram, Treasury};
pub use vesting::VestingContract;
pub use price_simulation::{PriceSimulation, CompletePriceSimulation}; // Re-export price simulation types
pub use nft::{NFTContract, NFT, NFTCollection, NFTMetadata, MarketplaceListing}; // Re-export NFT types
pub use liquidity_pool::{LiquidityPool, LiquidityPoolConfig, LiquidityPosition, PoolStats}; // Re-export liquidity pool types
pub use l2_rollup::{L2Rollup, L2Transaction, L2Block, L2Account, RollupConfig, RollupError}; // Re-export L2 rollup types
pub use l2_cross_chain::{L2CrossChainProtocol, CrossChainMessage}; // Re-export L2 cross-chain types
pub use l2_state_management::{L2StateManager, SparseMerkleTree, StateSnapshot, StateCheckpointManager}; // Re-export L2 state management types
pub use l2_batching::{TransactionBatch, BatchConfig, BatchSubmissionResult, BatchAggregator, BatchSubmitter}; // Re-export L2 batching types
pub use advanced_cryptography::{post_quantum, zero_knowledge, threshold_signatures}; // Re-export advanced cryptography types
pub use load_testing::{LoadTester, LoadTestConfig, LoadTestResult}; // Re-export load testing types

#[cfg(test)]
mod airdrop_test;

#[cfg(test)]
mod staking_test;

#[cfg(test)]
mod token_test;

#[cfg(test)]
mod treasury_test;

#[cfg(test)]
mod vesting_test;

#[cfg(test)]
mod team_allocation_test;

#[cfg(test)]
mod protection_features_test;

#[cfg(test)]
mod full_feature_integration_test;

#[cfg(test)]
mod investor_vesting_test;

#[cfg(test)]
mod nft_test; // Add NFT tests

#[cfg(test)]
mod liquidity_pool_test; // Add liquidity pool tests

#[cfg(test)]
mod yield_farming_integration_test; // Add yield farming integration tests

#[cfg(test)]
mod advanced_cryptography_test; // Add advanced cryptography tests

#[cfg(test)]
mod cryptography_integration_test; // Add cryptography integration tests

#[cfg(test)]
mod load_testing_test; // Add load testing tests