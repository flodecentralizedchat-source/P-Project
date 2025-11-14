use crate::airdrop::AirdropContract;
use crate::staking::StakingContract;
use crate::token::PProjectToken;
use crate::treasury::{LiquidityMiningProgram, Treasury};
use crate::vesting::VestingContract;

#[test]
fn test_dynamic_burn_rate() {
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Initialize some balances
    let allocations = vec![
        ("user1".to_string(), 1000000.0),
        ("user2".to_string(), 1000000.0),
    ];
    token.initialize_distribution(allocations);
    
    // Normal transfer with base burn rate
    let initial_supply = token.get_total_supply();
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_normal_transfer = token.get_total_supply();
    
    // Simulate high activity for user1
    for _ in 0..15 {
        token.transfer("user1", "user2", 1000.0).unwrap();
    }
    
    // Transfer with increased burn rate due to high activity
    let supply_before_high_activity = supply_after_normal_transfer;
    token.transfer("user1", "user2", 10000.0).unwrap();
    let supply_after_high_activity = token.get_total_supply();
    
    // The burn should be higher during high activity
    let normal_burn = initial_supply - supply_after_normal_transfer;
    let high_activity_burn = supply_before_high_activity - supply_after_high_activity;
    
    // High activity burn should be higher than normal burn
    assert!(high_activity_burn > normal_burn);
}

#[test]
fn test_treasury_buyback_integration() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);
    
    // Add funds to treasury
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();
    
    // Execute buyback
    let tokens_bought = treasury.execute_buyback(&mut token, 100000.0, 0.001).unwrap();
    assert_eq!(tokens_bought, 100000.0 / 0.001);
    
    // Verify treasury balance decreased
    assert_eq!(treasury.get_balance("USD"), 900000.0);
    
    // Verify buyback was recorded
    assert_eq!(treasury.get_total_buybacks(), 100000.0);
    assert_eq!(treasury.get_buyback_records().len(), 1);
}

#[test]
fn test_liquidity_mining_program_integration() {
    let mut program = LiquidityMiningProgram::new(
        "pool1".to_string(),
        "P".to_string(),
        1000.0, // 1000 tokens per day
        30,     // 30 days
        30000.0, // 30000 total tokens
    );
    
    // Add participants with different liquidity amounts
    program.add_participant("user1".to_string(), 10000.0);
    program.add_participant("user2".to_string(), 20000.0);
    program.add_participant("user3".to_string(), 30000.0);
    
    // Calculate rewards for different time periods
    let user1_rewards = program.calculate_rewards("user1", 7.0); // 7 days
    let user2_rewards = program.calculate_rewards("user2", 7.0); // 7 days
    let user3_rewards = program.calculate_rewards("user3", 7.0); // 7 days
    
    // Verify proportional rewards based on liquidity
    assert!(user2_rewards > user1_rewards);
    assert!(user3_rewards > user2_rewards);
    
    // User2 should get ~2x rewards of user1
    assert!((user2_rewards / user1_rewards - 2.0).abs() < 0.01);
    
    // User3 should get ~3x rewards of user1
    assert!((user3_rewards / user1_rewards - 3.0).abs() < 0.01);
}

#[test]
fn test_vesting_integration() {
    let total_vesting_tokens = 105000000.0; // 30% of 350M tokens
    let mut vesting_contract = VestingContract::new(total_vesting_tokens);
    
    let user_id = "team_member_1".to_string();
    let amount = 1000000.0; // 1M tokens
    let start_date = chrono::Utc::now().naive_utc();
    
    // Create team vesting schedule (12m cliff + 24m linear)
    let result = vesting_contract.create_team_vesting(user_id.clone(), amount, start_date);
    assert!(result.is_ok());
    
    // Check vesting schedule details
    let schedule = vesting_contract.get_vesting_schedule(&user_id);
    assert!(schedule.is_some());
    let schedule = schedule.unwrap();
    assert_eq!(schedule.total_amount, amount);
    assert_eq!(schedule.cliff_duration_months, 12);
    assert_eq!(schedule.vesting_duration_months, 24);
}

#[test]
fn test_staking_integration() {
    let mut staking_contract = StakingContract::new();
    let user_id = "user1".to_string();
    let amount = 100000.0;
    let duration_days = 365; // 1 year for maximum rewards
    
    // Stake tokens
    let result = staking_contract.stake_tokens(user_id.clone(), amount, duration_days);
    assert!(result.is_ok());
    
    // Check staking info
    let staking_info = staking_contract.get_staking_info(&user_id);
    assert!(staking_info.is_some());
    
    let staking_info = staking_info.unwrap();
    assert_eq!(staking_info.amount, amount);
    assert_eq!(staking_info.user_id, user_id);
}

#[test]
fn test_airdrop_integration() {
    let mut airdrop_contract = AirdropContract::new(52500000.0); // 15% of 350M tokens
    
    let recipients = vec![
        ("user1".to_string(), 10000.0),
        ("user2".to_string(), 20000.0),
        ("user3".to_string(), 15000.0),
    ];
    
    // Add recipients
    let result = airdrop_contract.add_recipients(recipients);
    assert!(result.is_ok());
    
    // Check status
    let status = airdrop_contract.get_status();
    assert_eq!(status.total_recipients, 3);
    assert_eq!(status.distributed_amount, 45000.0);
    
    // Claim airdrop
    let result = airdrop_contract.claim("user1");
    assert!(result.is_ok());
    
    let claimed_amount = result.unwrap();
    assert_eq!(claimed_amount, 10000.0);
    
    // Check that it's marked as claimed
    assert!(airdrop_contract.is_claimed("user1"));
}