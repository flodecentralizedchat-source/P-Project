use super::token::PProjectToken;
use super::treasury::{LiquidityMiningProgram, Treasury};

#[test]
fn test_treasury_creation() {
    let treasury = Treasury::new();
    assert_eq!(treasury.get_balance("USD"), 0.0);
    assert!(treasury.is_dao_controlled());
}

#[test]
fn test_treasury_funding() {
    let mut treasury = Treasury::new();

    // Add funds
    let result = treasury.add_funds("USD".to_string(), 1000000.0);
    assert!(result.is_ok());
    assert_eq!(treasury.get_balance("USD"), 1000000.0);
}

#[test]
fn test_treasury_allocation() {
    let mut treasury = Treasury::new();

    // Add funds first
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();

    // Allocate funds
    let result = treasury.allocate_funds(
        "Staking Rewards".to_string(),
        200000.0,
        "Staking reward pool".to_string(),
    );
    assert!(result.is_ok());
    assert_eq!(treasury.get_balance("USD"), 800000.0);
}

#[test]
fn test_treasury_buyback() {
    let mut treasury = Treasury::new();
    let mut token = PProjectToken::new(350000000.0, 0.01, 0.005);

    // Add funds first
    treasury.add_funds("USD".to_string(), 1000000.0).unwrap();

    // Execute buyback
    let result = treasury.execute_buyback(&mut token, 100000.0, 0.001);
    assert!(result.is_ok());
    assert_eq!(treasury.get_balance("USD"), 900000.0);
    assert_eq!(treasury.get_total_buybacks(), 100000.0);
}

#[test]
fn test_liquidity_mining_program() {
    let mut program = LiquidityMiningProgram::new(
        "pool1".to_string(),
        "P".to_string(),
        1000.0,  // 1000 tokens per day
        30,      // 30 days
        30000.0, // 30000 total tokens
    );

    // Add participants
    program.add_participant("user1".to_string(), 10000.0);
    program.add_participant("user2".to_string(), 20000.0);

    // Check total liquidity
    assert_eq!(program.get_total_liquidity(), 30000.0);

    // Check if program is active
    assert!(program.is_active());

    // Calculate rewards
    let user1_rewards = program.calculate_rewards("user1", 7.0); // 7 days
    let user2_rewards = program.calculate_rewards("user2", 7.0); // 7 days

    // User2 should get twice the rewards of user1 (twice the liquidity)
    assert!(user2_rewards > user1_rewards);
    assert!((user2_rewards / user1_rewards - 2.0).abs() < 0.001);
}
