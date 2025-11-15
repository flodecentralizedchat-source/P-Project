use chrono::{Duration, NaiveDateTime, Utc};
use p_project_core::models::StakingInfo;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Custom error types for staking operations
#[derive(Debug, Clone, PartialEq)]
pub enum StakingError {
    InvalidAmount,
    NoStakingInfo,
    StakingPositionExists,
    EmergencyWithdrawalsDisabled,
    DatabaseError(String),
    SerializationError(String),
}

impl std::fmt::Display for StakingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StakingError::InvalidAmount => write!(f, "Amount must be positive"),
            StakingError::NoStakingInfo => write!(f, "No staking info found for user"),
            StakingError::StakingPositionExists => {
                write!(f, "Target user already has a staking position")
            }
            StakingError::EmergencyWithdrawalsDisabled => {
                write!(f, "Emergency withdrawals are currently disabled")
            }
            StakingError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StakingError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for StakingError {}

// Staking tier with different APY rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingTier {
    pub name: String,
    pub min_amount: f64,
    pub duration_days: i64,
    pub apy_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRewardsConfig {
    pub total_rewards_pool: f64, // 17.5M tokens for staking rewards
    pub start_date: NaiveDateTime,
    pub year1_allocation: f64,    // 40% of total rewards
    pub year2_allocation: f64,    // 30% of total rewards
    pub year3_allocation: f64,    // 20% of total rewards
    pub year4_allocation: f64,    // 10% of total rewards
    pub distributed_rewards: f64, // Track distributed rewards
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    staking_infos: HashMap<String, StakingInfo>, // user_id -> staking info
    total_staked: Decimal,
    staking_tiers: Vec<StakingTier>, // Different staking tiers with APY rates
    emergency_withdrawals_enabled: bool, // Emergency withdrawal feature flag
    rewards_config: StakingRewardsConfig, // Staking rewards configuration
    // New fields for team staking incentives
    team_member_boost: f64, // Additional APY boost for team members
    team_member_list: HashMap<String, bool>, // user_id -> is_team_member
    // New fields for peace staking - rewards tied to donation events
    donation_events: HashMap<String, Vec<DonationEvent>>, // user_id -> donation events
    peace_staking_bonuses: HashMap<String, PeaceStakingBonus>, // user_id -> peace staking bonus info
}

// New struct for donation events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationEvent {
    pub event_id: String,
    pub user_id: String,
    pub donation_amount: f64,
    pub timestamp: NaiveDateTime,
    pub staking_bonus_multiplier: f64, // Multiplier for peace staking rewards
}

// New struct for peace staking bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceStakingBonus {
    pub user_id: String,
    pub total_bonus_earned: Decimal,
    pub last_calculated: NaiveDateTime,
}

impl StakingContract {
    pub fn new_with_rewards(total_rewards_pool: f64, start_date: NaiveDateTime) -> Self {
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
            // Special tier for team members
            StakingTier {
                name: "Team".to_string(),
                min_amount: 1000.0,
                duration_days: 180, // 6 months minimum for team members
                apy_rate: 0.25,     // 25% APY base rate
            },
        ];

        // Initialize rewards configuration with halving schedule
        let rewards_config = StakingRewardsConfig {
            total_rewards_pool,
            start_date,
            year1_allocation: total_rewards_pool * 0.4, // 40% in year 1
            year2_allocation: total_rewards_pool * 0.3, // 30% in year 2
            year3_allocation: total_rewards_pool * 0.2, // 20% in year 3
            year4_allocation: total_rewards_pool * 0.1, // 10% in year 4
            distributed_rewards: 0.0,
        };

        Self {
            staking_infos: HashMap::new(),
            total_staked: Decimal::ZERO,
            staking_tiers: tiers,
            emergency_withdrawals_enabled: false,
            rewards_config,
            team_member_boost: 0.05, // 5% additional APY for team members
            team_member_list: HashMap::new(), // Initialize team member list
            donation_events: HashMap::new(), // Initialize donation events
            peace_staking_bonuses: HashMap::new(), // Initialize peace staking bonuses
        }
    }

    pub fn new() -> Self {
        // Default constructor without specific rewards config
        let start_date = Utc::now().naive_utc();
        Self::new_with_rewards(17500000.0, start_date) // 17.5M tokens as per tokenomics
    }

    /// Get current year's reward allocation
    pub fn get_current_year_allocation(&self) -> f64 {
        let now = Utc::now().naive_utc();
        let elapsed_duration = now - self.rewards_config.start_date;
        let elapsed_years = elapsed_duration.num_days() / 365;

        match elapsed_years {
            0 => self.rewards_config.year1_allocation,
            1 => self.rewards_config.year2_allocation,
            2 => self.rewards_config.year3_allocation,
            3 => self.rewards_config.year4_allocation,
            _ => 0.0, // No rewards after year 4
        }
    }

    /// Get remaining rewards for current year
    pub fn get_remaining_yearly_rewards(&self) -> f64 {
        let current_year_allocation = self.get_current_year_allocation();
        let rewards_distributed_this_year = self.get_rewards_distributed_this_year();
        current_year_allocation - rewards_distributed_this_year
    }

    /// Get rewards distributed this year
    fn get_rewards_distributed_this_year(&self) -> f64 {
        // This is a simplified implementation
        // In a real implementation, you would track rewards by year
        self.rewards_config.distributed_rewards
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
    ) -> Result<String, StakingError> {
        if amount <= 0.0 {
            return Err(StakingError::InvalidAmount);
        }

        // Convert f64 to Decimal
        let amount_decimal = Decimal::from_f64(amount).ok_or(StakingError::InvalidAmount)?;

        // Determine staking tier based on amount and duration
        let tier_name = {
            let tier = self
                .determine_tier(amount, duration_days)
                .map_err(|_| StakingError::InvalidAmount)?;
            tier.name.clone()
        };

        let start_time = Utc::now().naive_utc();
        let end_time = start_time + Duration::days(duration_days);

        let staking_info = StakingInfo {
            user_id: user_id.clone(),
            amount: amount_decimal,
            start_time,
            end_time: Some(end_time),
            rewards_earned: Decimal::ZERO,
            tier_name: Some(tier_name.clone()),
            is_compounding: false,
        };

        self.staking_infos.insert(user_id.clone(), staking_info);
        self.total_staked += amount_decimal;

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
    pub fn unstake_tokens(&mut self, user_id: &str) -> Result<(f64, f64), StakingError> {
        // Returns (amount, rewards)
        let staking_info = self
            .staking_infos
            .remove(user_id)
            .ok_or(StakingError::NoStakingInfo)?;

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

        // Update distributed rewards tracking
        self.rewards_config.distributed_rewards += final_rewards;

        // Convert Decimal values back to f64 for return
        let amount_f64 = staking_info.amount.to_f64().unwrap_or(0.0);
        Ok((amount_f64, final_rewards))
    }

    /// Emergency withdrawal (with higher penalties)
    pub fn emergency_withdraw(&mut self, user_id: &str) -> Result<(f64, f64), StakingError> {
        // Returns (amount, penalties)
        if !self.emergency_withdrawals_enabled {
            return Err(StakingError::EmergencyWithdrawalsDisabled);
        }

        let staking_info = self
            .staking_infos
            .remove(user_id)
            .ok_or(StakingError::NoStakingInfo)?;

        // Higher penalty for emergency withdrawal (50% of staked amount + all rewards)
        let penalty_decimal = staking_info.amount * Decimal::from_f64(0.5).unwrap_or(Decimal::ZERO);
        let rewards = self.calculate_rewards(&staking_info);

        self.total_staked -= staking_info.amount;

        // Update distributed rewards tracking
        self.rewards_config.distributed_rewards += rewards;

        // Convert Decimal values back to f64 for return
        let amount_f64 = staking_info.amount.to_f64().unwrap_or(0.0);
        let penalty_f64 = penalty_decimal.to_f64().unwrap_or(0.0);
        Ok((amount_f64 - penalty_f64, rewards))
    }

    /// Transfer staking position to another user
    pub fn transfer_staking_position(
        &mut self,
        from_user_id: &str,
        to_user_id: String,
    ) -> Result<(), StakingError> {
        if !self.staking_infos.contains_key(from_user_id) {
            return Err(StakingError::NoStakingInfo);
        }

        if self.staking_infos.contains_key(&to_user_id) {
            return Err(StakingError::StakingPositionExists);
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
        let amount_f64 = staking_info.amount.to_f64().unwrap_or(0.0);
        let rewards = amount_f64 * (f64::exp(apy_rate * years) - 1.0);

        rewards
    }

    /// Get APY rate for a specific staking position with team member boost
    fn get_apy_for_staking_info(&self, staking_info: &StakingInfo) -> f64 {
        // If tier_name is available, find matching tier
        let base_apy = if let Some(tier_name) = &staking_info.tier_name {
            let mut found_apy = 0.05; // Default to basic 5% APY if no tier matches
            for tier in &self.staking_tiers {
                if &tier.name == tier_name {
                    found_apy = tier.apy_rate;
                    break;
                }
            }
            found_apy
        } else {
            0.05 // Default to basic 5% APY if no tier matches
        };

        // Apply team member boost if applicable
        if self.is_team_member(&staking_info.user_id) {
            base_apy + self.team_member_boost
        } else {
            base_apy
        }
    }

    /// Get staking info for a user
    pub fn get_staking_info(&self, user_id: &str) -> Option<&StakingInfo> {
        self.staking_infos.get(user_id)
    }

    /// Get total staked amount
    pub fn get_total_staked(&self) -> f64 {
        self.total_staked.to_f64().unwrap_or(0.0)
    }

    /// Compound rewards for a user (manually trigger compounding)
    pub fn compound_rewards(&mut self, user_id: &str) -> Result<f64, StakingError> {
        // Returns compounded amount
        let rewards = {
            let staking_info = self
                .staking_infos
                .get(user_id)
                .ok_or(StakingError::NoStakingInfo)?;
            self.calculate_rewards(staking_info)
        };

        let rewards_decimal = Decimal::from_f64(rewards).ok_or(StakingError::InvalidAmount)?;
        let staking_info = self
            .staking_infos
            .get_mut(user_id)
            .expect("Staking info must exist");

        staking_info.rewards_earned += rewards_decimal;
        staking_info.amount += rewards_decimal;
        staking_info.is_compounding = true;

        // Reset start time to now for new compounding period
        staking_info.start_time = Utc::now().naive_utc();

        // Update distributed rewards tracking
        self.rewards_config.distributed_rewards += rewards;

        Ok(rewards)
    }

    /// Enable compounding for a staking position
    pub fn enable_compounding(&mut self, user_id: &str) -> Result<(), StakingError> {
        let staking_info = self
            .staking_infos
            .get_mut(user_id)
            .ok_or(StakingError::NoStakingInfo)?;
        staking_info.is_compounding = true;
        Ok(())
    }

    /// Check if emergency withdrawals are enabled
    pub fn is_emergency_withdrawals_enabled(&self) -> bool {
        self.emergency_withdrawals_enabled
    }

    /// Get staking rewards configuration
    pub fn get_rewards_config(&self) -> &StakingRewardsConfig {
        &self.rewards_config
    }

    /// Add a user to the team member list for staking incentives
    pub fn add_team_member(&mut self, user_id: String) {
        self.team_member_list.insert(user_id, true);
    }

    /// Remove a user from the team member list
    pub fn remove_team_member(&mut self, user_id: &str) {
        self.team_member_list.remove(user_id);
    }

    /// Check if a user is a team member
    pub fn is_team_member(&self, user_id: &str) -> bool {
        *self.team_member_list.get(user_id).unwrap_or(&false)
    }

    /// Set team member APY boost
    pub fn set_team_member_boost(&mut self, boost: f64) {
        self.team_member_boost = boost;
    }

    /// Get team member APY boost
    pub fn get_team_member_boost(&self) -> f64 {
        self.team_member_boost
    }

    /// Record a donation event for peace staking rewards
    pub fn record_donation_event(
        &mut self,
        event_id: String,
        user_id: String,
        donation_amount: f64,
    ) -> Result<(), StakingError> {
        if donation_amount <= 0.0 {
            return Err(StakingError::InvalidAmount);
        }

        let timestamp = Utc::now().naive_utc();

        // Calculate bonus multiplier based on donation amount
        // Larger donations get higher multipliers
        let staking_bonus_multiplier = if donation_amount >= 1000.0 {
            2.0 // 2x bonus for large donations
        } else if donation_amount >= 100.0 {
            1.5 // 1.5x bonus for medium donations
        } else {
            1.2 // 1.2x bonus for small donations
        };

        let donation_event = DonationEvent {
            event_id,
            user_id: user_id.clone(),
            donation_amount,
            timestamp,
            staking_bonus_multiplier,
        };

        // Add the donation event to the user's list
        self.donation_events
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(donation_event);

        // Initialize or update peace staking bonus for the user
        if !self.peace_staking_bonuses.contains_key(&user_id) {
            self.peace_staking_bonuses.insert(
                user_id.clone(),
                PeaceStakingBonus {
                    user_id: user_id.clone(),
                    total_bonus_earned: Decimal::ZERO,
                    last_calculated: timestamp,
                },
            );
        }

        Ok(())
    }

    /// Calculate peace staking bonus for a user based on their donation events
    pub fn calculate_peace_staking_bonus(&mut self, user_id: &str) -> Result<f64, StakingError> {
        if !self.donation_events.contains_key(user_id) {
            return Ok(0.0); // No donation events, no bonus
        }

        let donation_events = self.donation_events.get(user_id).unwrap();
        let mut total_bonus = 0.0;
        let now = Utc::now().naive_utc();

        // Calculate bonus based on donation events
        for event in donation_events {
            // Calculate time since donation (in days)
            let duration = now - event.timestamp;
            let days_since_donation = duration.num_days() as f64;

            // Bonus decreases over time (linear decay over 30 days)
            let time_decay_factor = if days_since_donation > 30.0 {
                0.0 // No bonus after 30 days
            } else {
                1.0 - (days_since_donation / 30.0) // Linear decay
            };

            // Calculate bonus: donation amount * multiplier * time decay
            let event_bonus =
                event.donation_amount * event.staking_bonus_multiplier * time_decay_factor * 0.01; // 1% base rate
            total_bonus += event_bonus;
        }

        let total_bonus_decimal =
            Decimal::from_f64(total_bonus).ok_or(StakingError::InvalidAmount)?;

        // Update the user's total bonus earned
        if let Some(bonus_info) = self.peace_staking_bonuses.get_mut(user_id) {
            bonus_info.total_bonus_earned += total_bonus_decimal;
            bonus_info.last_calculated = now;
        } else {
            // Create new bonus info if it doesn't exist
            self.peace_staking_bonuses.insert(
                user_id.to_string(),
                PeaceStakingBonus {
                    user_id: user_id.to_string(),
                    total_bonus_earned: total_bonus_decimal,
                    last_calculated: now,
                },
            );
        }

        Ok(total_bonus)
    }

    /// Get donation events for a user
    pub fn get_donation_events_for_staker(&self, user_id: &str) -> Option<&Vec<DonationEvent>> {
        self.donation_events.get(user_id)
    }

    /// Get peace staking bonus info for a user
    pub fn get_peace_staking_bonus(&self, user_id: &str) -> Option<&PeaceStakingBonus> {
        self.peace_staking_bonuses.get(user_id)
    }

    /// Get all donation events
    pub fn get_all_donation_events(&self) -> &HashMap<String, Vec<DonationEvent>> {
        &self.donation_events
    }

    /// Get all peace staking bonuses
    pub fn get_all_peace_staking_bonuses(&self) -> &HashMap<String, PeaceStakingBonus> {
        &self.peace_staking_bonuses
    }

    /// Calculate projected rewards for a staking position
    pub fn calculate_projected_rewards(
        &self,
        amount: f64,
        duration_days: i64,
        tier_name: Option<&str>,
    ) -> f64 {
        // Determine APY based on tier
        let apy_rate = if let Some(tier_name) = tier_name {
            let mut found_apy = 0.05; // Default to basic 5% APY if no tier matches
            for tier in &self.staking_tiers {
                if tier.name == tier_name {
                    found_apy = tier.apy_rate;
                    break;
                }
            }
            found_apy
        } else {
            // Find the best matching tier based on amount and duration
            let mut best_apy = 0.05; // Default to basic 5% APY
            for tier in &self.staking_tiers {
                if amount >= tier.min_amount && duration_days >= tier.duration_days {
                    if tier.apy_rate > best_apy {
                        best_apy = tier.apy_rate;
                    }
                }
            }
            best_apy
        };

        // Compound interest calculation: A = P(1 + r/n)^(nt)
        // For simplicity, we'll use continuous compounding: A = Pe^(rt)
        let years = duration_days as f64 / 365.0;
        let rewards = amount * (f64::exp(apy_rate * years) - 1.0);

        rewards
    }

    /// Get all staking tiers
    pub fn get_all_staking_tiers(&self) -> &Vec<StakingTier> {
        &self.staking_tiers
    }
}
