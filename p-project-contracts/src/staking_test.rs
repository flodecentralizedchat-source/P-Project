use super::staking::StakingContract;

#[test]
fn test_staking_contract_creation() {
    let staking_contract = StakingContract::new();
    assert_eq!(staking_contract.get_total_staked(), 0.0);
}

#[test]
fn test_staking_functionality() {
    let mut staking_contract = StakingContract::new();
    let user_id = "user1".to_string();
    let amount = 1000.0;
    let duration_days = 30;
    
    // Test staking
    let result = staking_contract.stake_tokens(user_id.clone(), amount, duration_days);
    assert!(result.is_ok());
    
    // Check total staked
    assert_eq!(staking_contract.get_total_staked(), amount);
    
    // Check staking info
    let staking_info = staking_contract.get_staking_info(&user_id);
    assert!(staking_info.is_some());
    
    let staking_info = staking_info.unwrap();
    assert_eq!(staking_info.amount, amount);
    assert_eq!(staking_info.user_id, user_id);
}

#[test]
fn test_unstaking_functionality() {
    let mut staking_contract = StakingContract::new();
    let user_id = "user1".to_string();
    let amount = 1000.0;
    let duration_days = 30;
    
    // Stake tokens first
    let _ = staking_contract.stake_tokens(user_id.clone(), amount, duration_days);
    
    // Test unstaking
    let result = staking_contract.unstake_tokens(&user_id);
    assert!(result.is_ok());
    
    let (unstaked_amount, rewards) = result.unwrap();
    assert_eq!(unstaked_amount, amount);
    assert!(rewards >= 0.0);
    
    // Check total staked is now zero
    assert_eq!(staking_contract.get_total_staked(), 0.0);
}

#[test]
fn test_staking_rewards_config() {
    let start_date = chrono::Utc::now().naive_utc();
    let total_rewards_pool = 17500000.0; // 17.5M tokens as per tokenomics
    let staking_contract = StakingContract::new_with_rewards(total_rewards_pool, start_date);
    
    let rewards_config = staking_contract.get_rewards_config();
    assert_eq!(rewards_config.total_rewards_pool, total_rewards_pool);
    assert_eq!(rewards_config.start_date, start_date);
    assert_eq!(rewards_config.year1_allocation, total_rewards_pool * 0.4); // 40%
    assert_eq!(rewards_config.year2_allocation, total_rewards_pool * 0.3); // 30%
    assert_eq!(rewards_config.year3_allocation, total_rewards_pool * 0.2); // 20%
    assert_eq!(rewards_config.year4_allocation, total_rewards_pool * 0.1); // 10%
}