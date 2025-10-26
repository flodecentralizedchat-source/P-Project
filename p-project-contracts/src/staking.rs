use p_project_core::models::StakingInfo;
use std::collections::HashMap;

pub struct StakingContract {
    staking_infos: HashMap<String, StakingInfo>, // user_id -> staking info
    total_staked: f64,
    reward_rate: f64, // annual reward rate
}

impl StakingContract {
    pub fn new(reward_rate: f64) -> Self {
        Self {
            staking_infos: HashMap::new(),
            total_staked: 0.0,
            reward_rate,
        }
    }
    
    /// Stake tokens for a user
    pub fn stake_tokens(&mut self, user_id: String, amount: f64, duration_days: i64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        
        let start_time = chrono::Utc::now().naive_utc();
        let end_time = start_time + chrono::Duration::days(duration_days);
        
        let staking_info = StakingInfo {
            user_id: user_id.clone(),
            amount,
            start_time,
            end_time: Some(end_time),
            rewards_earned: 0.0,
        };
        
        self.staking_infos.insert(user_id, staking_info);
        self.total_staked += amount;
        
        Ok(())
    }
    
    /// Unstake tokens for a user
    pub fn unstake_tokens(&mut self, user_id: &str) -> Result<f64, String> {
        let staking_info = self.staking_infos.remove(user_id).ok_or("No staking info found for user")?;
        
        let rewards = self.calculate_rewards(&staking_info);
        self.total_staked -= staking_info.amount;
        
        Ok(staking_info.amount + rewards)
    }
    
    /// Calculate rewards for a staking position
    fn calculate_rewards(&self, staking_info: &StakingInfo) -> f64 {
        let duration = staking_info.end_time.unwrap_or_else(|| chrono::Utc::now().naive_utc()) - staking_info.start_time;
        let days = duration.num_days() as f64;
        let rewards = staking_info.amount * self.reward_rate * (days / 365.0);
        rewards
    }
    
    /// Get staking info for a user
    pub fn get_staking_info(&self, user_id: &str) -> Option<&StakingInfo> {
        self.staking_infos.get(user_id)
    }
    
    /// Get total staked amount
    pub fn get_total_staked(&self) -> f64 {
        self.total_staked
    }
}