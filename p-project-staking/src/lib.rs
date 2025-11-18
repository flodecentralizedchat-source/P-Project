use p_project_contracts::staking::{StakingContract, StakingError};
use p_project_core::database::MySqlDatabase;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Minimal async DB abstraction for staking persistence used by the service.
#[async_trait::async_trait]
pub trait StakeDb: Send + Sync {
    async fn stake_tokens(
        &self,
        stake_id: &str,
        user_id: &str,
        amount: Decimal,
        duration_days: i64,
    ) -> Result<p_project_core::models::StakingInfo, String>;

    async fn unstake_tokens(
        &self,
        user_id: &str,
        stake_id: Option<&str>,
    ) -> Result<p_project_core::models::StakingInfo, String>;

    async fn save_staking_state(&self, state_data: &str) -> Result<(), String>;
}

#[async_trait::async_trait]
impl StakeDb for MySqlDatabase {
    async fn stake_tokens(
        &self,
        stake_id: &str,
        user_id: &str,
        amount: Decimal,
        duration_days: i64,
    ) -> Result<p_project_core::models::StakingInfo, String> {
        self.stake_tokens(stake_id, user_id, amount, duration_days)
            .await
            .map_err(|e| e.to_string())
    }

    async fn unstake_tokens(
        &self,
        user_id: &str,
        stake_id: Option<&str>,
    ) -> Result<p_project_core::models::StakingInfo, String> {
        self.unstake_tokens(user_id, stake_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn save_staking_state(&self, state_data: &str) -> Result<(), String> {
        self.save_staking_state(state_data)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Service that coordinates contract logic with persistence
pub struct StakingService<D: StakeDb> {
    staking_contract: StakingContract,
    db: D,
}

impl<D: StakeDb> StakingService<D> {
    pub fn new(staking_contract: StakingContract, db: D) -> Self {
        Self {
            staking_contract,
            db,
        }
    }

    /// Stake tokens for a user (persists to DB and updates contract state)
    pub async fn stake_tokens(
        &mut self,
        user_id: String,
        amount: f64,
        duration_days: i64,
    ) -> Result<String, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        if duration_days <= 0 {
            return Err("Duration must be positive".to_string());
        }

        let stake_id = uuid::Uuid::new_v4().to_string();
        let amount_decimal =
            Decimal::from_f64(amount).ok_or_else(|| "invalid_amount".to_string())?;

        // Persist first to ensure balances are updated atomically at the DB level
        let _db_info = self
            .db
            .stake_tokens(&stake_id, &user_id, amount_decimal, duration_days)
            .await?;

        // Update in-memory contract state
        let tier_name = self
            .staking_contract
            .stake_tokens(user_id, amount, duration_days)
            .map_err(|e| e.to_string())?;

        // Save contract state snapshot
        let state = serde_json::to_string(&self.staking_contract).map_err(|e| e.to_string())?;
        self.db.save_staking_state(&state).await?;

        Ok(tier_name)
    }

    /// Unstake tokens for a user (updates contract and DB)
    pub async fn unstake_tokens(&mut self, user_id: &str) -> Result<(f64, f64), String> {
        // Compute rewards and update contract state
        let (amount, rewards) =
            self.staking_contract
                .unstake_tokens(user_id)
                .map_err(|e| match e {
                    StakingError::NoStakingInfo => "stake_not_found".to_string(),
                    _ => e.to_string(),
                })?;

        // Persist DB side
        let _ = self.db.unstake_tokens(user_id, None).await?;

        // Save contract state snapshot
        let state = serde_json::to_string(&self.staking_contract).map_err(|e| e.to_string())?;
        self.db.save_staking_state(&state).await?;

        Ok((amount, rewards))
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

/// Convenience alias for the common MySQL-backed service
pub type MySqlStakingService = StakingService<MySqlDatabase>;
