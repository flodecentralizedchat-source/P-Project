pub mod advanced_cryptography; // Add the new advanced cryptography module
pub mod advanced_verification; // Add the new advanced verification module
pub mod airdrop;
pub mod audit_registry; // Audit metadata registry
pub mod charity; // Add the new charity allocator module
pub mod community_liquidity; // Community liquidity incentives
pub mod compliance; // Compliance and regulatory controls
pub mod comprehensive_test_suite; // Add the new comprehensive test suite module
pub mod comprehensive_verification; // Add the new comprehensive verification module
pub mod cross_chain_liquidity; // Cross-chain liquidity orchestration
pub mod dex_listing; // DEX listing orchestration helpers
pub mod router; // Router with automatic liquidity provision
pub mod formal_verification; // Add the new formal verification module
pub mod l2_batching; // Add the new L2 batching module
pub mod l2_cross_chain; // Add the new L2 cross-chain module
pub mod l2_model_checking; // Add the new L2 model checking module
pub mod l2_rollup; // Add the new L2 rollup module
pub mod l2_state_management; // Add the new L2 state management module
pub mod liquidity_pool; // Add the new liquidity pool module
pub mod load_testing;
pub mod metaverse;
pub mod nft; // Add the new NFT module
pub mod ownership;
pub mod price_simulation; // Add the new price simulation module
pub mod savings_vault;
pub mod security_compliance; // Additional security controls
pub mod stable_liquidity_pool; // Add the new stable liquidity pool module
pub mod staking;
pub mod supply_chain;
pub mod theorem_proving; // Add the new theorem proving module
pub mod token;
pub mod treasury;
pub mod vesting; // Add the new load testing module // Ownable + renounce mechanics

// Re-export the main contract types
pub use advanced_cryptography::{post_quantum, threshold_signatures, zero_knowledge}; // Re-export advanced cryptography types
pub use airdrop::{AirdropContract, MerkleTree};
pub use charity::{
    AidVoucher, Allocation, AuditEvent, CharityAllocator, CharityError, CreditTransaction,
    CrowdfundCampaign, DashboardSummary, DistributionRule, DonationRecord, DonorReputation,
    LeaderboardEntry, NGOImpactRecord, PeaceReliefCredit, ProofOfPeaceBadge, NGO,
}; // Re-export charity allocator types
pub use l2_batching::{
    BatchAggregator, BatchConfig, BatchSubmissionResult, BatchSubmitter, TransactionBatch,
}; // Re-export L2 batching types
pub use l2_cross_chain::{CrossChainMessage, L2CrossChainProtocol}; // Re-export L2 cross-chain types
pub use l2_rollup::{L2Account, L2Block, L2Rollup, L2Transaction, RollupConfig, RollupError}; // Re-export L2 rollup types
pub use l2_state_management::{
    L2StateManager, SparseMerkleTree, StateCheckpointManager, StateSnapshot,
}; // Re-export L2 state management types
pub use liquidity_pool::{
    LiquidityMechanisms, LiquidityPool, LiquidityPoolConfig, LiquidityPosition, PoolStats,
}; // Re-export liquidity pool types
pub use load_testing::{LoadTestConfig, LoadTestResult, LoadTester};
pub use metaverse::{Building, BuildingType, LandParcel, MetaverseError, PeaceIsland};
pub use nft::{MarketplaceListing, NFTCollection, NFTContract, NFTMetadata, NFT}; // Re-export NFT types
pub use price_simulation::{CompletePriceSimulation, PriceSimulation}; // Re-export price simulation types
pub use savings_vault::{SavingsConfig, SavingsError, SavingsVault};
pub use stable_liquidity_pool::{StableLiquidityPool, StablePoolConfig}; // Re-export stable LP types
pub use staking::StakingContract;
pub use supply_chain::{
    AidShipment, AlertSeverity, AntiCorruptionAlert, DonationItem, LogisticsEvent, SupplyCategory,
    SupplyChainError, SupplyChainTracker,
};
pub use token::PProjectToken;
pub use treasury::{LiquidityMiningProgram, Treasury};

#[cfg(test)]
mod router_test;
pub use vesting::VestingContract; // Re-export load testing types

#[cfg(test)]
mod airdrop_test;

#[cfg(test)]
mod staking_test;

#[cfg(test)]
mod token_test;

#[cfg(test)]
mod treasury_test;

#[cfg(test)]
mod treasury_presale_test;

#[cfg(test)]
mod vesting_test;

#[cfg(test)]
mod charity_test; // Add charity allocator tests

#[cfg(test)]
mod charity_accounting_test;

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
mod savings_vault_test;

#[cfg(test)]
mod charity_stable_payout_test; // Add NGO stable payout tests

#[cfg(test)]
mod stable_liquidity_pool_test; // Add stable liquidity pool tests

#[cfg(test)]
mod cross_chain_liquidity_test; // Add cross-chain liquidity tests

#[cfg(test)]
mod dex_listing_test; // Add DEX listing orchestration tests

#[cfg(test)]
mod supply_chain_test;

#[cfg(test)]
mod metaverse_test;

#[cfg(test)]
mod compliance_test;

#[cfg(test)]
mod audit_registry_test;

#[cfg(test)]
mod ownership_test;

#[cfg(test)]
mod advanced_cryptography_test; // Add advanced cryptography tests

#[cfg(test)]
mod cryptography_integration_test; // Add cryptography integration tests

#[cfg(test)]
mod load_testing_test; // Add load testing tests

#[cfg(test)]
mod social_proof_test; // Add social proof & impact tests
