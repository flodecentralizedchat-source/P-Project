//! P-Project Staking Program for Solana
//! 
//! This program implements staking functionality for the P-Project token on Solana.
//! Users can stake their P tokens and earn rewards over time.

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Program entrypoint
entrypoint!(process_instruction);

/// Program state handler
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("P-Project Staking Program entrypoint");
    
    // TODO: Implement staking logic
    // This is a placeholder for the actual implementation
    
    Ok(())
}

/// Staking pool state
#[derive(Debug)]
pub struct StakingPool {
    /// Total amount of tokens staked in the pool
    pub total_staked: u64,
    /// Total rewards distributed
    pub total_rewards: u64,
    /// Reward rate per epoch
    pub reward_rate: u64,
    /// Last time rewards were updated
    pub last_update: i64,
}

/// User staking position
#[derive(Debug)]
pub struct StakingPosition {
    /// Amount of tokens staked by the user
    pub staked_amount: u64,
    /// Pending rewards for the user
    pub pending_rewards: u64,
    /// Time when the user started staking
    pub start_time: i64,
    /// Duration of the stake lockup period
    pub lockup_duration: i64,
}

impl StakingPool {
    /// Create a new staking pool
    pub fn new(reward_rate: u64) -> Self {
        Self {
            total_staked: 0,
            total_rewards: 0,
            reward_rate,
            last_update: 0,
        }
    }
    
    /// Add tokens to the staking pool
    pub fn stake(&mut self, amount: u64) -> Result<(), ProgramError> {
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        self.total_staked += amount;
        msg!("Staked {} tokens", amount);
        
        Ok(())
    }
    
    /// Remove tokens from the staking pool
    pub fn unstake(&mut self, amount: u64) -> Result<(), ProgramError> {
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }
        
        if amount > self.total_staked {
            return Err(ProgramError::InsufficientFunds);
        }
        
        self.total_staked -= amount;
        msg!("Unstaked {} tokens", amount);
        
        Ok(())
    }
    
    /// Calculate rewards for the pool
    pub fn calculate_rewards(&self, current_time: i64) -> u64 {
        let time_elapsed = current_time - self.last_update;
        if time_elapsed <= 0 {
            return 0;
        }
        
        // Simple reward calculation: reward_rate * time_elapsed
        self.reward_rate * (time_elapsed as u64)
    }
    
    /// Update pool with new rewards
    pub fn update_rewards(&mut self, current_time: i64) {
        let new_rewards = self.calculate_rewards(current_time);
        self.total_rewards += new_rewards;
        self.last_update = current_time;
        msg!("Updated rewards: {} new rewards added", new_rewards);
    }
}

impl StakingPosition {
    /// Create a new staking position
    pub fn new(staked_amount: u64, start_time: i64, lockup_duration: i64) -> Self {
        Self {
            staked_amount,
            pending_rewards: 0,
            start_time,
            lockup_duration,
        }
    }
    
    /// Add to staked amount
    pub fn add_stake(&mut self, amount: u64) {
        self.staked_amount += amount;
    }
    
    /// Remove from staked amount
    pub fn remove_stake(&mut self, amount: u64) -> Result<(), ProgramError> {
        if amount > self.staked_amount {
            return Err(ProgramError::InsufficientFunds);
        }
        
        self.staked_amount -= amount;
        Ok(())
    }
    
    /// Add pending rewards
    pub fn add_rewards(&mut self, rewards: u64) {
        self.pending_rewards += rewards;
    }
    
    /// Claim rewards
    pub fn claim_rewards(&mut self) -> u64 {
        let rewards = self.pending_rewards;
        self.pending_rewards = 0;
        rewards
    }
    
    /// Check if the stake is unlocked
    pub fn is_unlocked(&self, current_time: i64) -> bool {
        current_time >= self.start_time + self.lockup_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_pool_creation() {
        let reward_rate = 100;
        let pool = StakingPool::new(reward_rate);
        
        assert_eq!(pool.total_staked, 0);
        assert_eq!(pool.total_rewards, 0);
        assert_eq!(pool.reward_rate, reward_rate);
        assert_eq!(pool.last_update, 0);
    }

    #[test]
    fn test_staking_pool_stake() {
        let mut pool = StakingPool::new(100);
        let stake_amount = 1000;
        
        assert!(pool.stake(stake_amount).is_ok());
        assert_eq!(pool.total_staked, stake_amount);
    }

    #[test]
    fn test_staking_pool_unstake() {
        let mut pool = StakingPool::new(100);
        let stake_amount = 1000;
        let unstake_amount = 500;
        
        // First stake some tokens
        assert!(pool.stake(stake_amount).is_ok());
        
        // Then unstake some tokens
        assert!(pool.unstake(unstake_amount).is_ok());
        assert_eq!(pool.total_staked, stake_amount - unstake_amount);
    }

    #[test]
    fn test_staking_pool_unstake_insufficient() {
        let mut pool = StakingPool::new(100);
        let stake_amount = 1000;
        let unstake_amount = 1500;
        
        // First stake some tokens
        assert!(pool.stake(stake_amount).is_ok());
        
        // Try to unstake more than staked
        assert!(pool.unstake(unstake_amount).is_err());
    }

    #[test]
    fn test_staking_pool_rewards() {
        let mut pool = StakingPool::new(100);
        let start_time = 1000;
        let current_time = 2000;
        
        pool.last_update = start_time;
        let rewards = pool.calculate_rewards(current_time);
        
        assert_eq!(rewards, 100 * (current_time - start_time) as u64);
    }

    #[test]
    fn test_staking_position_creation() {
        let staked_amount = 1000;
        let start_time = 1000;
        let lockup_duration = 3600;
        
        let position = StakingPosition::new(staked_amount, start_time, lockup_duration);
        
        assert_eq!(position.staked_amount, staked_amount);
        assert_eq!(position.pending_rewards, 0);
        assert_eq!(position.start_time, start_time);
        assert_eq!(position.lockup_duration, lockup_duration);
    }

    #[test]
    fn test_staking_position_add_stake() {
        let mut position = StakingPosition::new(1000, 1000, 3600);
        let additional_stake = 500;
        
        position.add_stake(additional_stake);
        assert_eq!(position.staked_amount, 1500);
    }

    #[test]
    fn test_staking_position_remove_stake() {
        let mut position = StakingPosition::new(1000, 1000, 3600);
        let remove_amount = 300;
        
        assert!(position.remove_stake(remove_amount).is_ok());
        assert_eq!(position.staked_amount, 700);
    }

    #[test]
    fn test_staking_position_remove_stake_insufficient() {
        let mut position = StakingPosition::new(1000, 1000, 3600);
        let remove_amount = 1500;
        
        assert!(position.remove_stake(remove_amount).is_err());
    }

    #[test]
    fn test_staking_position_rewards() {
        let mut position = StakingPosition::new(1000, 1000, 3600);
        let rewards = 100;
        
        position.add_rewards(rewards);
        assert_eq!(position.pending_rewards, rewards);
        
        let claimed = position.claim_rewards();
        assert_eq!(claimed, rewards);
        assert_eq!(position.pending_rewards, 0);
    }

    #[test]
    fn test_staking_position_unlocked() {
        let start_time = 1000;
        let lockup_duration = 3600;
        let position = StakingPosition::new(1000, start_time, lockup_duration);
        
        // Before lockup period ends
        assert!(!position.is_unlocked(start_time + 1000));
        
        // After lockup period ends
        assert!(position.is_unlocked(start_time + 4000));
    }
}