//! Tests for staking contract with database integration

#[cfg(test)]
mod tests {
    use crate::staking::{StakingContract, StakingError};

    #[test]
    fn test_staking_contract_creation() {
        let staking = StakingContract::new();
        assert_eq!(staking.get_total_staked(), 0.0);
        assert!(!staking.is_emergency_withdrawals_enabled());
    }

    #[test]
    fn test_stake_tokens_success() {
        let mut staking = StakingContract::new();
        let result = staking.stake_tokens("user1".to_string(), 1000.0, 30);
        assert!(result.is_ok());
        assert_eq!(staking.get_total_staked(), 1000.0);
    }

    #[test]
    fn test_stake_tokens_invalid_amount() {
        let mut staking = StakingContract::new();
        let result = staking.stake_tokens("user1".to_string(), -100.0, 30);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::InvalidAmount);

        let result = staking.stake_tokens("user1".to_string(), 0.0, 30);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::InvalidAmount);
    }

    #[test]
    fn test_unstake_tokens_success() {
        let mut staking = StakingContract::new();
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();

        let result = staking.unstake_tokens("user1");
        assert!(result.is_ok());
        let (amount, _rewards) = result.unwrap();
        assert_eq!(amount, 1000.0);
        assert_eq!(staking.get_total_staked(), 0.0);
    }

    #[test]
    fn test_unstake_tokens_no_staking_info() {
        let mut staking = StakingContract::new();
        let result = staking.unstake_tokens("user1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::NoStakingInfo);
    }

    #[test]
    fn test_emergency_withdrawal() {
        let mut staking = StakingContract::new();
        staking.set_emergency_withdrawals(true);
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();

        let result = staking.emergency_withdraw("user1");
        assert!(result.is_ok());
        assert_eq!(staking.get_total_staked(), 0.0);
    }

    #[test]
    fn test_emergency_withdrawal_disabled() {
        let mut staking = StakingContract::new();
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();

        let result = staking.emergency_withdraw("user1");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            StakingError::EmergencyWithdrawalsDisabled
        );
    }

    #[test]
    fn test_transfer_staking_position() {
        let mut staking = StakingContract::new();
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();

        let result = staking.transfer_staking_position("user1", "user2".to_string());
        assert!(result.is_ok());

        assert!(staking.get_staking_info("user1").is_none());
        assert!(staking.get_staking_info("user2").is_some());
    }

    #[test]
    fn test_transfer_staking_position_no_info() {
        let mut staking = StakingContract::new();

        let result = staking.transfer_staking_position("user1", "user2".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::NoStakingInfo);
    }

    #[test]
    fn test_transfer_staking_position_exists() {
        let mut staking = StakingContract::new();
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();
        staking
            .stake_tokens("user2".to_string(), 500.0, 30)
            .unwrap();

        let result = staking.transfer_staking_position("user1", "user2".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::StakingPositionExists);
    }

    #[test]
    fn test_compound_rewards() {
        let mut staking = StakingContract::new();
        staking
            .stake_tokens("user1".to_string(), 1000.0, 30)
            .unwrap();

        let result = staking.compound_rewards("user1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_rewards_no_info() {
        let mut staking = StakingContract::new();

        let result = staking.compound_rewards("user1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StakingError::NoStakingInfo);
    }
}
