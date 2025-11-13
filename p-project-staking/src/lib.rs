use p_project_contracts::staking::{StakingContract, StakingError};
use p_project_core::database::MySqlDatabase;

pub struct StakingService {
    staking_contract: StakingContract,
    db: MySqlDatabase,
}

impl StakingService {
    pub fn new(staking_contract: StakingContract, db: MySqlDatabase) -> Self {
        Self {
            staking_contract,
            db,
        }
    }

    /// Stake tokens for a user
    pub fn stake_tokens(
        &mut self,
        user_id: String,
        amount: f64,
        duration_days: i64,
    ) -> Result<String, String> {
        self.staking_contract
            .stake_tokens(user_id, amount, duration_days)
            .map_err(|e| e.to_string())
    }

    /// Unstake tokens for a user
    pub fn unstake_tokens(&mut self, user_id: &str) -> Result<(f64, f64), String> {
        self.staking_contract
            .unstake_tokens(user_id)
            .map_err(|e| e.to_string())
    }

    /// Get staking info for a user
    pub fn get_staking_info(&self, user_id: &str) -> Option<p_project_core::models::StakingInfo> {
        self.staking_contract.get_staking_info(user_id).cloned()
    }

    /// Get total staked amount
    pub fn get_total_staked(&self) -> f64 {
        self.staking_contract.get_total_staked()
    }
}