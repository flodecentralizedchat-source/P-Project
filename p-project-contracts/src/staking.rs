use chrono::{Duration, Utc};
use p_project_core::models::StakingInfo;
use std::collections::HashMap;

// Staking tier with different APY rates
#[derive(Debug, Clone)]
pub struct StakingTier {
    pub name: String,
    pub min_amount: f64,
    pub duration_days: i64,
    pub apy_rate: f64,
}

pub struct StakingContract {
    staking_infos: HashMap<String, StakingInfo>, // user_id -> staking info
    total_staked: f64,
    staking_tiers: Vec<StakingTier>, // Different staking tiers with APY rates
    emergency_withdrawals_enabled: bool, // Emergency withdrawal feature flag
}

impl StakingContract {
    pub fn new() -> Self {
        // Initialize with default staking tiers
        let tiers = vec![
            StakingTier {
                name: "Basic".to_string(),
                min_amount: 100.0,
                duration_days: 30,
                apy_rate: 0.05, // 5% APY
            },
            StakingTier {
                name: "Silver".to_string(),
                min_amount: 1000.0,
                duration_days: 90,
                apy_rate: 0.10, // 10% APY
            },
            StakingTier {
                name: "Gold".to_string(),
                min_amount: 10000.0,
                duration_days: 365,
                apy_rate: 0.20, // 20% APY
            },
        ];

        Self {
            staking_infos: HashMap::new(),
            total_staked: 0.0,
            staking_tiers: tiers,
            emergency_withdrawals_enabled: false,
        }
    }

    /// Enable or disable emergency withdrawals
    pub fn set_emergency_withdrawals(&mut self, enabled: bool) {
        self.emergency_withdrawals_enabled = enabled;
    }

    /// Get staking tiers
    pub fn get_staking_tiers(&self) -> &Vec<StakingTier> {
        &self.staking_tiers
    }

    /// Add a new staking tier
    pub fn add_staking_tier(&mut self, tier: StakingTier) {
        self.staking_tiers.push(tier);
    }

    /// Stake tokens for a user with tier selection
    pub fn stake_tokens(
        &mut self,
        user_id: String,
        amount: f64,
        duration_days: i64,
    ) -> Result<String, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        // Determine staking tier based on amount and duration
        let tier_name = {
            let tier = self.determine_tier(amount, duration_days)?;
            tier.name.clone()
        };

        let start_time = Utc::now().naive_utc();
        let end_time = start_time + Duration::days(duration_days);

        let staking_info = StakingInfo {
            user_id: user_id.clone(),
            amount,
            start_time,
            end_time: Some(end_time),
            rewards_earned: 0.0,
            tier_name: Some(tier_name.clone()),
            is_compounding: false,
        };

        self.staking_infos.insert(user_id.clone(), staking_info);
        self.total_staked += amount;

        Ok(tier_name)
    }

    /// Determine staking tier based on amount and duration
    fn determine_tier(&self, amount: f64, duration_days: i64) -> Result<&StakingTier, String> {
        // Find the best matching tier
        let mut best_tier: Option<&StakingTier> = None;

        for tier in &self.staking_tiers {
            if amount >= tier.min_amount && duration_days >= tier.duration_days {
                match best_tier {
                    None => best_tier = Some(tier),
                    Some(current) => {
                        // Select tier with higher APY or higher amount requirement
                        if tier.apy_rate > current.apy_rate
                            || (tier.apy_rate == current.apy_rate
                                && tier.min_amount > current.min_amount)
                        {
                            best_tier = Some(tier);
                        }
                    }
                }
            }
        }

        best_tier.ok_or(
            "No matching staking tier found for the specified amount and duration".to_string(),
        )
    }

    /// Unstake tokens for a user with early unstaking penalties
    pub fn unstake_tokens(&mut self, user_id: &str) -> Result<(f64, f64), String> {
        // Returns (amount, rewards)
        let staking_info = self
            .staking_infos
            .remove(user_id)
            .ok_or("No staking info found for user")?;

        let now = Utc::now().naive_utc();
        let is_early_unstake = match staking_info.end_time {
            Some(end_time) => now < end_time,
            None => false,
        };

        let rewards = self.calculate_rewards(&staking_info);
        let penalty = if is_early_unstake {
            rewards * 0.25 // 25% penalty on rewards for early unstaking
        } else {
            0.0
        };

        let final_rewards = rewards - penalty;
        self.total_staked -= staking_info.amount;

        Ok((staking_info.amount, final_rewards))
    }

    /// Emergency withdrawal (with higher penalties)
    pub fn emergency_withdraw(&mut self, user_id: &str) -> Result<(f64, f64), String> {
        // Returns (amount, penalties)
        if !self.emergency_withdrawals_enabled {
            return Err("Emergency withdrawals are currently disabled".to_string());
        }

        let staking_info = self
            .staking_infos
            .remove(user_id)
            .ok_or("No staking info found for user")?;

        // Higher penalty for emergency withdrawal (50% of staked amount + all rewards)
        let penalty = staking_info.amount * 0.5;
        let rewards = self.calculate_rewards(&staking_info);

        self.total_staked -= staking_info.amount;

        Ok((staking_info.amount - penalty, rewards))
    }

    /// Transfer staking position to another user
    pub fn transfer_staking_position(
        &mut self,
        from_user_id: &str,
        to_user_id: String,
    ) -> Result<(), String> {
        if !self.staking_infos.contains_key(from_user_id) {
            return Err("No staking position found for the source user".to_string());
        }

        if self.staking_infos.contains_key(&to_user_id) {
            return Err("Target user already has a staking position".to_string());
        }

        let staking_info = self.staking_infos.remove(from_user_id).unwrap();
        let updated_info = StakingInfo {
            user_id: to_user_id.clone(),
            amount: staking_info.amount,
            start_time: staking_info.start_time,
            end_time: staking_info.end_time,
            rewards_earned: staking_info.rewards_earned,
            tier_name: staking_info.tier_name,
            is_compounding: staking_info.is_compounding,
        };

        self.staking_infos.insert(to_user_id, updated_info);
        Ok(())
    }

    /// Calculate compounded rewards for a staking position
    fn calculate_rewards(&self, staking_info: &StakingInfo) -> f64 {
        let duration = staking_info
            .end_time
            .unwrap_or_else(|| Utc::now().naive_utc())
            - staking_info.start_time;
        let days = duration.num_days() as f64;

        // Determine APY based on tier
        let apy_rate = self.get_apy_for_staking_info(staking_info);

        // Compound interest calculation: A = P(1 + r/n)^(nt)
        // For simplicity, we'll use continuous compounding: A = Pe^(rt)
        let years = days / 365.0;
        let rewards = staking_info.amount * (f64::exp(apy_rate * years) - 1.0);

        rewards
    }

    /// Get APY rate for a specific staking position
    fn get_apy_for_staking_info(&self, staking_info: &StakingInfo) -> f64 {
        // If tier_name is available, find matching tier
        if let Some(tier_name) = &staking_info.tier_name {
            for tier in &self.staking_tiers {
                if &tier.name == tier_name {
                    return tier.apy_rate;
                }
            }
        }

        // Default to basic 5% APY if no tier matches
        0.05
    }

    /// Get staking info for a user
    pub fn get_staking_info(&self, user_id: &str) -> Option<&StakingInfo> {
        self.staking_infos.get(user_id)
    }

    /// Get total staked amount
    pub fn get_total_staked(&self) -> f64 {
        self.total_staked
    }

    /// Compound rewards for a user (manually trigger compounding)
    pub fn compound_rewards(&mut self, user_id: &str) -> Result<f64, String> {
        // Returns compounded amount
        let rewards = {
            let staking_info = self
                .staking_infos
                .get(user_id)
                .ok_or("No staking info found for user")?;
            self.calculate_rewards(staking_info)
        };

        let staking_info = self
            .staking_infos
            .get_mut(user_id)
            .expect("Staking info must exist");

        staking_info.rewards_earned += rewards;
        staking_info.amount += rewards;
        staking_info.is_compounding = true;

        // Reset start time to now for new compounding period
        staking_info.start_time = Utc::now().naive_utc();

        Ok(rewards)
    }

    /// Enable compounding for a staking position
    pub fn enable_compounding(&mut self, user_id: &str) -> Result<(), String> {
        let staking_info = self
            .staking_infos
            .get_mut(user_id)
            .ok_or("No staking info found for user")?;
        staking_info.is_compounding = true;
        Ok(())
    }
}
